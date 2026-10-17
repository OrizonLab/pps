//! Tests d'intégration du moteur de soin : cercles, membres, consentements,
//! journal d'audit et étanchéité entre familles (PostgreSQL réel avec le rôle
//! applicatif, `DATABASE_URL` requis).

// Fichier de tests : les fonctions utilitaires peuvent échouer franchement.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod commun;

use std::collections::BTreeSet;

use commun::*;
use pps_db::{TxCercle, audit, membres};
use pps_domaine::audit::{Acteur, Action, verifier_chaine};
use pps_domaine::consentement::{Categorie, OrigineRetrait, RefusConsentement};
use pps_domaine::ids::{IdAppel, IdCercle};
use pps_soin::{ErreurSoin, autorisation};
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

const MIGRATIONS: &sqlx::migrate::Migrator = &pps_db::MIGRATIONS;

#[sqlx::test(migrator = "MIGRATIONS")]
async fn creer_un_cercle_le_met_en_preparation_et_journalise(
    options: PgPoolOptions,
    connexion: PgConnectOptions,
) {
    let bdd = pool_applicatif(options, connexion).await;
    let sophie = utilisateur(&bdd).await;

    let cercle = moteur(&bdd)
        .creer_cercle(nouveau_cercle(sophie))
        .await
        .unwrap();

    let (mut tx, membre) = TxCercle::pour_membre(&bdd, sophie, cercle).await.unwrap();
    assert_eq!(membre.role, pps_domaine::membre::RoleMembre::Organisateur);
    let journal = audit::lire_journal(&mut tx).await.unwrap();
    assert_eq!(journal.len(), 2);
    assert_eq!(journal[0].action, Action::CercleCree);
    assert_eq!(journal[0].acteur, Acteur::Utilisateur(sophie));
    assert_eq!(journal[1].action, Action::MembreAjoute);
    verifier_chaine(&journal).unwrap();

    // Aucun accord tant que le senior n'a pas accepté le service.
    let erreur = moteur(&bdd)
        .accorder_consentement(cercle, membre.id, Categorie::CheckIn, accord_par_appel())
        .await
        .unwrap_err();
    assert!(matches!(
        erreur,
        ErreurSoin::Consentement(RefusConsentement::CercleNonActif)
    ));
}

#[sqlx::test(migrator = "MIGRATIONS")]
async fn seul_l_organisateur_ajoute_des_proches(
    options: PgPoolOptions,
    connexion: PgConnectOptions,
) {
    let bdd = pool_applicatif(options, connexion).await;
    let (sophie, lucas, marc) = (
        utilisateur(&bdd).await,
        utilisateur(&bdd).await,
        utilisateur(&bdd).await,
    );
    let moteur = moteur(&bdd);
    let cercle = moteur.creer_cercle(nouveau_cercle(sophie)).await.unwrap();

    moteur
        .ajouter_proche(sophie, cercle, proche(lucas, "Lucas"))
        .await
        .unwrap();

    // Un proche ne peut pas ajouter de membre.
    let erreur = moteur
        .ajouter_proche(lucas, cercle, proche(marc, "Marc"))
        .await
        .unwrap_err();
    assert!(matches!(erreur, ErreurSoin::ReserveeOrganisateur));

    // Pas de doublon.
    let erreur = moteur
        .ajouter_proche(sophie, cercle, proche(lucas, "Lucas encore"))
        .await
        .unwrap_err();
    assert!(matches!(erreur, ErreurSoin::DejaMembre));

    // Un inconnu n'a aucun accès.
    let erreur = moteur
        .ajouter_proche(marc, cercle, proche(marc, "Marc"))
        .await
        .unwrap_err();
    assert!(matches!(erreur, ErreurSoin::AccesRefuse));
}

#[sqlx::test(migrator = "MIGRATIONS")]
async fn le_senior_decide_qui_voit_quoi(options: PgPoolOptions, connexion: PgConnectOptions) {
    let bdd = pool_applicatif(options, connexion).await;
    let (sophie, lucas) = (utilisateur(&bdd).await, utilisateur(&bdd).await);
    let moteur = moteur(&bdd);
    let cercle = moteur.creer_cercle(nouveau_cercle(sophie)).await.unwrap();
    let membre_lucas = moteur
        .ajouter_proche(sophie, cercle, proche(lucas, "Lucas"))
        .await
        .unwrap();
    let membre_sophie = moteur.qui_voit_quoi(cercle).await.unwrap()[0].membre;

    moteur
        .activer_cercle(cercle, accord_par_appel())
        .await
        .unwrap();
    // Le rôle d'organisateur ne donne aucun accès aux données de santé.
    assert!(
        moteur
            .mes_autorisations(sophie, cercle)
            .await
            .unwrap()
            .is_empty()
    );

    for categorie in [Categorie::PrisesMedicaments, Categorie::Alertes] {
        moteur
            .accorder_consentement(cercle, membre_sophie, categorie, accord_par_appel())
            .await
            .unwrap();
    }
    moteur
        .accorder_consentement(cercle, membre_lucas, Categorie::CheckIn, accord_par_appel())
        .await
        .unwrap();

    assert_eq!(
        moteur.mes_autorisations(sophie, cercle).await.unwrap(),
        BTreeSet::from([Categorie::PrisesMedicaments, Categorie::Alertes])
    );
    assert_eq!(
        moteur.mes_autorisations(lucas, cercle).await.unwrap(),
        BTreeSet::from([Categorie::CheckIn])
    );
    let visibilites = moteur.qui_voit_quoi(cercle).await.unwrap();
    assert_eq!(visibilites.len(), 2);
    assert_eq!(visibilites[1].nom_affiche.as_str(), "Lucas");
    assert_eq!(
        visibilites[1].categories,
        BTreeSet::from([Categorie::CheckIn])
    );

    // Retrait par le senior, au téléphone.
    moteur
        .retirer_consentement(
            cercle,
            membre_sophie,
            Categorie::Alertes,
            OrigineRetrait::AppelEntrant(IdAppel::nouveau()),
        )
        .await
        .unwrap();
    // Lucas renonce lui-même.
    moteur
        .renoncer(lucas, cercle, Categorie::CheckIn)
        .await
        .unwrap();
    // Le senior ne peut pas « renoncer » à la place d'un membre.
    let erreur = moteur
        .retirer_consentement(
            cercle,
            membre_sophie,
            Categorie::PrisesMedicaments,
            OrigineRetrait::RenonciationDuMembre,
        )
        .await
        .unwrap_err();
    assert!(matches!(
        erreur,
        ErreurSoin::Consentement(RefusConsentement::OrigineIncoherente)
    ));

    assert_eq!(
        moteur.mes_autorisations(sophie, cercle).await.unwrap(),
        BTreeSet::from([Categorie::PrisesMedicaments])
    );
    assert!(
        moteur
            .mes_autorisations(lucas, cercle)
            .await
            .unwrap()
            .is_empty()
    );

    // Tout est journalisé, et la chaîne est intacte.
    let mut tx = TxCercle::pour_moteur_de_soin(&bdd, cercle).await.unwrap();
    let journal = audit::lire_journal(&mut tx).await.unwrap();
    verifier_chaine(&journal).unwrap();
    let actions: Vec<Action> = journal.iter().map(|e| e.action).collect();
    assert_eq!(
        actions,
        [
            Action::CercleCree,
            Action::MembreAjoute,
            Action::MembreAjoute,
            Action::CercleActive,
            Action::ConsentementAccorde,
            Action::ConsentementAccorde,
            Action::ConsentementAccorde,
            Action::ConsentementRetire,
            Action::ConsentementRetire,
        ]
    );
    assert_eq!(journal[3].acteur, Acteur::Senior);
    assert_eq!(journal[8].acteur, Acteur::Utilisateur(lucas));
}

#[sqlx::test(migrator = "MIGRATIONS")]
async fn un_membre_retire_perd_tout_acces(options: PgPoolOptions, connexion: PgConnectOptions) {
    let bdd = pool_applicatif(options, connexion).await;
    let (sophie, lucas) = (utilisateur(&bdd).await, utilisateur(&bdd).await);
    let moteur = moteur(&bdd);
    let cercle = moteur.creer_cercle(nouveau_cercle(sophie)).await.unwrap();
    let membre_lucas = moteur
        .ajouter_proche(sophie, cercle, proche(lucas, "Lucas"))
        .await
        .unwrap();
    moteur
        .activer_cercle(cercle, accord_par_appel())
        .await
        .unwrap();
    moteur
        .accorder_consentement(cercle, membre_lucas, Categorie::CheckIn, accord_par_appel())
        .await
        .unwrap();

    moteur
        .retirer_membre(sophie, cercle, membre_lucas)
        .await
        .unwrap();

    let erreur = moteur.mes_autorisations(lucas, cercle).await.unwrap_err();
    assert!(matches!(erreur, ErreurSoin::AccesRefuse));
    let mut tx = TxCercle::pour_moteur_de_soin(&bdd, cercle).await.unwrap();
    let lu = membres::lire(&mut tx, membre_lucas).await.unwrap().unwrap();
    assert!(!lu.actif);
    // Même avec un ancien accord, un membre retiré ne voit rien.
    assert!(
        !autorisation(&mut tx, &lu, Categorie::CheckIn)
            .await
            .unwrap()
    );
    assert!(
        moteur
            .qui_voit_quoi(cercle)
            .await
            .unwrap()
            .iter()
            .all(|v| v.membre != membre_lucas)
    );

    // L'organisateur ne peut pas être retiré.
    let organisateur = moteur.qui_voit_quoi(cercle).await.unwrap()[0].membre;
    let erreur = moteur
        .retirer_membre(sophie, cercle, organisateur)
        .await
        .unwrap_err();
    assert!(matches!(erreur, ErreurSoin::OrganisateurNonRetirable));
}

#[sqlx::test(migrator = "MIGRATIONS")]
async fn un_cercle_ne_voit_jamais_les_donnees_d_un_autre(
    options: PgPoolOptions,
    connexion: PgConnectOptions,
) {
    let bdd = pool_applicatif(options, connexion).await;
    let (sophie, paul) = (utilisateur(&bdd).await, utilisateur(&bdd).await);
    let moteur = moteur(&bdd);
    let cercle_a = moteur.creer_cercle(nouveau_cercle(sophie)).await.unwrap();
    let cercle_b = moteur.creer_cercle(nouveau_cercle(paul)).await.unwrap();
    let membre_paul = moteur.qui_voit_quoi(cercle_b).await.unwrap()[0].membre;

    // Connaître l'identifiant d'un autre cercle ne suffit pas.
    let erreur = moteur
        .mes_autorisations(sophie, cercle_b)
        .await
        .unwrap_err();
    assert!(matches!(erreur, ErreurSoin::AccesRefuse));
    assert!(matches!(
        TxCercle::pour_membre(&bdd, sophie, cercle_b).await,
        Err(pps_db::ErreurAcces::NonMembre)
    ));

    // Dans la transaction du cercle A, les lignes du cercle B sont invisibles.
    let mut tx = TxCercle::pour_moteur_de_soin(&bdd, cercle_a).await.unwrap();
    assert!(membres::lire(&mut tx, membre_paul).await.unwrap().is_none());
    assert_eq!(membres::lister_actifs(&mut tx).await.unwrap().len(), 1);
    assert_eq!(audit::lire_journal(&mut tx).await.unwrap().len(), 2);

    // Un cercle inexistant est refusé comme un cercle étranger.
    let erreur = moteur
        .mes_autorisations(sophie, IdCercle::nouveau())
        .await
        .unwrap_err();
    assert!(matches!(erreur, ErreurSoin::AccesRefuse));
}

#[sqlx::test(migrator = "MIGRATIONS")]
async fn les_historiques_refusent_modification_et_suppression(
    options: PgPoolOptions,
    connexion: PgConnectOptions,
) {
    let bdd: PgPool = pool_applicatif(options, connexion).await;
    let sophie = utilisateur(&bdd).await;
    let cercle = moteur(&bdd)
        .creer_cercle(nouveau_cercle(sophie))
        .await
        .unwrap();
    let contexte = cercle.to_string();

    let mut tx = bdd.begin().await.unwrap();
    sqlx::query_scalar!("SELECT set_config('app.cercle_id', $1, true)", contexte)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    let erreur = sqlx::query!("UPDATE app.audit SET action = 'falsifiee'")
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(
        erreur.as_database_error().unwrap().code().as_deref(),
        Some("42501")
    );
    tx.rollback().await.unwrap();

    let mut tx = bdd.begin().await.unwrap();
    sqlx::query_scalar!("SELECT set_config('app.cercle_id', $1, true)", contexte)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    let erreur = sqlx::query!("DELETE FROM app.consentement_evenement")
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(
        erreur.as_database_error().unwrap().code().as_deref(),
        Some("42501")
    );
    tx.rollback().await.unwrap();
}
