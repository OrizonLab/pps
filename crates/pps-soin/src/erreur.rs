//! Erreurs du moteur de soin.

use pps_db::{ErreurAcces, ErreurDonnees};
use pps_domaine::cercle::TransitionInterdite;
use pps_domaine::consentement::RefusConsentement;

/// Refus ou échec d'un cas d'usage. Les messages ne contiennent aucune
/// donnée personnelle.
#[derive(Debug, thiserror::Error)]
pub enum ErreurSoin {
    #[error("accès refusé à ce cercle")]
    AccesRefuse,
    #[error("action réservée à l'organisateur du cercle")]
    ReserveeOrganisateur,
    #[error("membre introuvable dans ce cercle")]
    MembreIntrouvable,
    #[error("cet utilisateur est déjà membre du cercle")]
    DejaMembre,
    #[error("l'organisateur ne peut pas être retiré du cercle")]
    OrganisateurNonRetirable,
    #[error(transparent)]
    Transition(#[from] TransitionInterdite),
    #[error(transparent)]
    Consentement(#[from] RefusConsentement),
    #[error(transparent)]
    Donnees(#[from] ErreurDonnees),
}

impl From<ErreurAcces> for ErreurSoin {
    fn from(erreur: ErreurAcces) -> Self {
        match erreur {
            ErreurAcces::NonMembre => Self::AccesRefuse,
            ErreurAcces::Donnees(erreur) => Self::Donnees(erreur),
        }
    }
}
