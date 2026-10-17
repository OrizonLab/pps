//! Cas d'usage du cercle de soin : création, membres, activation par le
//! senior, consentements.
//!
//! Chaque cas d'usage s'exécute dans une seule transaction limitée au cercle
//! et journalise ce qu'il fait. Les actions du senior (activation, accords,
//! retraits) viennent des appels téléphoniques (incrément 7).

use std::collections::BTreeSet;

use pps_db::membres::{MembreLu, NouveauMembre};
use pps_db::{TxCercle, audit, cercles, consentements, membres};
use pps_domaine::audit::{Acteur, Action, NouvelleEntree, Objet};
use pps_domaine::cercle::{FuseauHoraire, IdentiteSenior, TexteCourt};
use pps_domaine::consentement::{
    AuteurRetrait, Categorie, EtatConsentements, EvenementConsentement, Nature, OrigineAccord,
    OrigineRetrait, verifier_accord, verifier_retrait,
};
use pps_domaine::horodatage::Horloge;
use pps_domaine::ids::{IdCercle, IdEvenementConsentement, IdMembre, IdUtilisateur};
use pps_domaine::membre::{Lien, RoleMembre};
use sqlx::PgPool;

use crate::erreur::ErreurSoin;

/// Données de création d'un cercle, saisies par l'organisateur. Aucune
/// donnée de santé à ce stade.
#[derive(Debug, Clone)]
pub struct NouveauCercle {
    pub senior: IdentiteSenior,
    pub fuseau: FuseauHoraire,
    pub organisateur: IdUtilisateur,
    pub nom_affiche_organisateur: TexteCourt,
    pub lien_organisateur: Lien,
}

/// Un proche à ajouter au cercle.
#[derive(Debug, Clone)]
pub struct NouveauProche {
    pub utilisateur: IdUtilisateur,
    pub lien: Lien,
    pub nom_affiche: TexteCourt,
}

/// Ce qu'un membre peut voir : la réponse à « qui voit quoi ? ».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibiliteMembre {
    pub membre: IdMembre,
    pub nom_affiche: TexteCourt,
    pub lien: Lien,
    pub categories: BTreeSet<Categorie>,
}

/// Moteur des cercles de soin.
#[derive(Debug, Clone)]
pub struct MoteurCercle<H> {
    bdd: PgPool,
    horloge: H,
}

impl<H: Horloge> MoteurCercle<H> {
    pub fn new(bdd: PgPool, horloge: H) -> Self {
        Self { bdd, horloge }
    }

    /// Crée un cercle en préparation, dont le créateur est l'organisateur.
    pub async fn creer_cercle(&self, nouveau: NouveauCercle) -> Result<IdCercle, ErreurSoin> {
        let maintenant = self.horloge.maintenant();
        let cercle = IdCercle::nouveau();
        let acteur = Acteur::Utilisateur(nouveau.organisateur);
        let mut tx = TxCercle::pour_nouveau_cercle(&self.bdd, nouveau.organisateur, cercle).await?;

        cercles::inserer(&mut tx, &nouveau.senior, &nouveau.fuseau, maintenant).await?;
        let organisateur = membres::inserer(
            &mut tx,
            &NouveauMembre {
                utilisateur: nouveau.organisateur,
                role: RoleMembre::Organisateur,
                lien: nouveau.lien_organisateur,
                nom_affiche: nouveau.nom_affiche_organisateur,
            },
            maintenant,
        )
        .await?;
        journaliser(
            &mut tx,
            acteur,
            Action::CercleCree,
            Objet::Cercle(cercle),
            &self.horloge,
        )
        .await?;
        journaliser(
            &mut tx,
            acteur,
            Action::MembreAjoute,
            Objet::Membre(organisateur),
            &self.horloge,
        )
        .await?;
        tx.valider().await?;
        Ok(cercle)
    }

    /// L'organisateur ajoute un proche. Le proche ne voit aucune donnée de
    /// santé tant que le senior n'a pas consenti pour lui.
    pub async fn ajouter_proche(
        &self,
        acteur: IdUtilisateur,
        cercle: IdCercle,
        proche: NouveauProche,
    ) -> Result<IdMembre, ErreurSoin> {
        let (mut tx, demandeur) = TxCercle::pour_membre(&self.bdd, acteur, cercle).await?;
        if !demandeur.role.peut_gerer_les_membres() {
            return Err(ErreurSoin::ReserveeOrganisateur);
        }
        cercles::lire_pour_mise_a_jour(&mut tx).await?;
        if membres::actif_par_utilisateur(&mut tx, proche.utilisateur)
            .await?
            .is_some()
        {
            return Err(ErreurSoin::DejaMembre);
        }
        let membre = membres::inserer(
            &mut tx,
            &NouveauMembre {
                utilisateur: proche.utilisateur,
                role: RoleMembre::Proche,
                lien: proche.lien,
                nom_affiche: proche.nom_affiche,
            },
            self.horloge.maintenant(),
        )
        .await?;
        journaliser(
            &mut tx,
            Acteur::Utilisateur(acteur),
            Action::MembreAjoute,
            Objet::Membre(membre),
            &self.horloge,
        )
        .await?;
        tx.valider().await?;
        Ok(membre)
    }

    /// Retire un membre : l'organisateur peut retirer un proche, et chacun
    /// peut quitter le cercle. Un membre retiré perd immédiatement tout accès.
    pub async fn retirer_membre(
        &self,
        acteur: IdUtilisateur,
        cercle: IdCercle,
        membre: IdMembre,
    ) -> Result<(), ErreurSoin> {
        let (mut tx, demandeur) = TxCercle::pour_membre(&self.bdd, acteur, cercle).await?;
        if demandeur.id != membre && !demandeur.role.peut_gerer_les_membres() {
            return Err(ErreurSoin::ReserveeOrganisateur);
        }
        cercles::lire_pour_mise_a_jour(&mut tx).await?;
        let cible = membres::lire(&mut tx, membre)
            .await?
            .filter(|m| m.actif)
            .ok_or(ErreurSoin::MembreIntrouvable)?;
        if cible.role == RoleMembre::Organisateur {
            return Err(ErreurSoin::OrganisateurNonRetirable);
        }
        membres::retirer(&mut tx, membre, self.horloge.maintenant()).await?;
        journaliser(
            &mut tx,
            Acteur::Utilisateur(acteur),
            Action::MembreRetire,
            Objet::Membre(membre),
            &self.horloge,
        )
        .await?;
        tx.valider().await?;
        Ok(())
    }

    /// Le senior accepte le service lors de l'appel de bienvenue (appel émis
    /// par PPS) : le cercle devient actif.
    pub async fn activer_cercle(
        &self,
        cercle: IdCercle,
        // Le type garantit que l'accord vient d'un appel émis par PPS.
        _origine: OrigineAccord,
    ) -> Result<(), ErreurSoin> {
        let mut tx = TxCercle::pour_moteur_de_soin(&self.bdd, cercle).await?;
        let lu = cercles::lire_pour_mise_a_jour(&mut tx).await?;
        let statut = lu.statut.activer()?;
        cercles::changer_statut(&mut tx, statut, self.horloge.maintenant()).await?;
        journaliser(
            &mut tx,
            Acteur::Senior,
            Action::CercleActive,
            Objet::Cercle(cercle),
            &self.horloge,
        )
        .await?;
        tx.valider().await?;
        Ok(())
    }

    /// Le senior accorde à un membre l'accès à une catégorie, lors d'un appel
    /// émis par PPS.
    pub async fn accorder_consentement(
        &self,
        cercle: IdCercle,
        membre: IdMembre,
        categorie: Categorie,
        origine: OrigineAccord,
    ) -> Result<IdEvenementConsentement, ErreurSoin> {
        let mut tx = TxCercle::pour_moteur_de_soin(&self.bdd, cercle).await?;
        let lu = cercles::lire_pour_mise_a_jour(&mut tx).await?;
        let cible = membres::lire(&mut tx, membre)
            .await?
            .ok_or(ErreurSoin::MembreIntrouvable)?;
        verifier_accord(lu.statut, cible.actif)?;
        let evenement = self
            .enregistrer(
                &mut tx,
                membre,
                categorie,
                Nature::Accord(origine),
                Acteur::Senior,
            )
            .await?;
        tx.valider().await?;
        Ok(evenement)
    }

    /// Le senior retire un accès, par n'importe quel canal.
    pub async fn retirer_consentement(
        &self,
        cercle: IdCercle,
        membre: IdMembre,
        categorie: Categorie,
        origine: OrigineRetrait,
    ) -> Result<IdEvenementConsentement, ErreurSoin> {
        verifier_retrait(AuteurRetrait::Senior, origine)?;
        let mut tx = TxCercle::pour_moteur_de_soin(&self.bdd, cercle).await?;
        cercles::lire_pour_mise_a_jour(&mut tx).await?;
        membres::lire(&mut tx, membre)
            .await?
            .ok_or(ErreurSoin::MembreIntrouvable)?;
        let evenement = self
            .enregistrer(
                &mut tx,
                membre,
                categorie,
                Nature::Retrait(origine),
                Acteur::Senior,
            )
            .await?;
        tx.valider().await?;
        Ok(evenement)
    }

    /// Un membre renonce lui-même à son accès à une catégorie.
    pub async fn renoncer(
        &self,
        utilisateur: IdUtilisateur,
        cercle: IdCercle,
        categorie: Categorie,
    ) -> Result<IdEvenementConsentement, ErreurSoin> {
        let origine = OrigineRetrait::RenonciationDuMembre;
        verifier_retrait(AuteurRetrait::LeMembreLuiMeme, origine)?;
        let (mut tx, membre) = TxCercle::pour_membre(&self.bdd, utilisateur, cercle).await?;
        cercles::lire_pour_mise_a_jour(&mut tx).await?;
        let evenement = self
            .enregistrer(
                &mut tx,
                membre.id,
                categorie,
                Nature::Retrait(origine),
                Acteur::Utilisateur(utilisateur),
            )
            .await?;
        tx.valider().await?;
        Ok(evenement)
    }

    /// « Qui voit quoi ? » : la réponse donnée au senior, membre par membre.
    pub async fn qui_voit_quoi(
        &self,
        cercle: IdCercle,
    ) -> Result<Vec<VisibiliteMembre>, ErreurSoin> {
        let mut tx = TxCercle::pour_moteur_de_soin(&self.bdd, cercle).await?;
        let etat = EtatConsentements::depuis_evenements(&consentements::lister(&mut tx).await?);
        let visibilites = membres::lister_actifs(&mut tx)
            .await?
            .into_iter()
            .map(|m| VisibiliteMembre {
                categories: etat.categories_de(m.id),
                membre: m.id,
                nom_affiche: m.nom_affiche,
                lien: m.lien,
            })
            .collect();
        tx.valider().await?;
        Ok(visibilites)
    }

    /// Ce que l'utilisateur connecté peut voir dans ce cercle.
    pub async fn mes_autorisations(
        &self,
        utilisateur: IdUtilisateur,
        cercle: IdCercle,
    ) -> Result<BTreeSet<Categorie>, ErreurSoin> {
        let (mut tx, membre) = TxCercle::pour_membre(&self.bdd, utilisateur, cercle).await?;
        let etat = EtatConsentements::depuis_evenements(&consentements::lister(&mut tx).await?);
        tx.valider().await?;
        Ok(etat.categories_de(membre.id))
    }

    async fn enregistrer(
        &self,
        tx: &mut TxCercle,
        membre: IdMembre,
        categorie: Categorie,
        nature: Nature,
        acteur: Acteur,
    ) -> Result<IdEvenementConsentement, ErreurSoin> {
        let evenement = EvenementConsentement {
            id: IdEvenementConsentement::nouveau(),
            membre,
            categorie,
            nature,
            survenu_le: self.horloge.maintenant(),
        };
        consentements::inserer(tx, &evenement).await?;
        let action = match nature {
            Nature::Accord(_) => Action::ConsentementAccorde,
            Nature::Retrait(_) => Action::ConsentementRetire,
        };
        journaliser(
            tx,
            acteur,
            action,
            Objet::Consentement(evenement.id),
            &self.horloge,
        )
        .await?;
        Ok(evenement.id)
    }
}

/// Un membre peut-il voir cette catégorie ? Il doit être actif **et** avoir
/// l'accord du senior. À appeler, dans la transaction du cercle, avant toute
/// lecture d'une donnée de santé (CLAUDE.md, principe 2).
pub async fn autorisation(
    tx: &mut TxCercle,
    membre: &MembreLu,
    categorie: Categorie,
) -> Result<bool, ErreurSoin> {
    if !membre.actif {
        return Ok(false);
    }
    let etat = EtatConsentements::depuis_evenements(&consentements::lister(tx).await?);
    Ok(etat.autorise(membre.id, categorie))
}

async fn journaliser(
    tx: &mut TxCercle,
    acteur: Acteur,
    action: Action,
    objet: Objet,
    horloge: &impl Horloge,
) -> Result<(), ErreurSoin> {
    audit::journaliser(
        tx,
        NouvelleEntree {
            acteur,
            action,
            objet,
            survenu_le: horloge.maintenant(),
        },
    )
    .await?;
    Ok(())
}
