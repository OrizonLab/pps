//! Outils partagés par les tests d'intégration.

// Fichier de tests : les fonctions utilitaires peuvent échouer franchement.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use jiff::Timestamp;
use pps_db::utilisateurs;
use pps_domaine::cercle::{FuseauHoraire, IdentiteSenior, TexteCourt};
use pps_domaine::consentement::OrigineAccord;
use pps_domaine::horodatage::Horloge;
use pps_domaine::ids::{IdAppel, IdUtilisateur};
use pps_domaine::membre::Lien;
use pps_soin::{MoteurCercle, NouveauCercle, NouveauProche};
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

/// Pool connecté avec le rôle applicatif `pps_api` (comme en production) :
/// la Row-Level Security s'applique réellement.
pub(crate) async fn pool_applicatif(options: PgPoolOptions, connexion: PgConnectOptions) -> PgPool {
    options
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query!("SET ROLE pps_api").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with(connexion)
        .await
        .unwrap()
}

/// Horloge fixe et lisible pour les tests.
#[derive(Debug, Clone, Copy)]
pub(crate) struct HorlogeFixe(pub(crate) Timestamp);

impl Horloge for HorlogeFixe {
    fn maintenant(&self) -> Timestamp {
        self.0
    }
}

pub(crate) fn horloge() -> HorlogeFixe {
    HorlogeFixe("2026-10-17T08:30:00.123456789Z".parse().unwrap())
}

pub(crate) fn moteur(bdd: &PgPool) -> MoteurCercle<HorlogeFixe> {
    MoteurCercle::new(bdd.clone(), horloge())
}

pub(crate) async fn utilisateur(bdd: &PgPool) -> IdUtilisateur {
    utilisateurs::creer(bdd, horloge().0).await.unwrap()
}

pub(crate) fn texte(valeur: &str) -> TexteCourt {
    TexteCourt::nouveau(valeur).unwrap()
}

pub(crate) fn nouveau_cercle(organisateur: IdUtilisateur) -> NouveauCercle {
    NouveauCercle {
        senior: IdentiteSenior {
            prenom: texte("Jeanne"),
            appellation: texte("Madame Martin"),
            vouvoiement: true,
        },
        fuseau: FuseauHoraire::nouveau("Europe/Paris").unwrap(),
        organisateur,
        nom_affiche_organisateur: texte("Sophie"),
        lien_organisateur: Lien::Enfant,
    }
}

pub(crate) fn proche(utilisateur: IdUtilisateur, nom: &str) -> NouveauProche {
    NouveauProche {
        utilisateur,
        lien: Lien::PetitEnfant,
        nom_affiche: texte(nom),
    }
}

pub(crate) fn accord_par_appel() -> OrigineAccord {
    OrigineAccord::AppelSortant(IdAppel::nouveau())
}
