//! Erreurs d'accès aux données.

use pps_domaine::audit::ErreurAudit;
use pps_domaine::cercle::ErreurValeur;

/// Erreur lors d'une lecture ou d'une écriture.
#[derive(Debug, thiserror::Error)]
pub enum ErreurDonnees {
    #[error("erreur de base de données")]
    Bdd(#[from] sqlx::Error),
    /// Une valeur relue en base ne respecte pas les règles métier : la base
    /// a été modifiée hors de l'application, ou un bug l'a corrompue.
    #[error("donnée incohérente en base")]
    Incoherente(#[from] ErreurValeur),
    #[error(transparent)]
    Audit(#[from] ErreurAudit),
}

/// Erreur à l'ouverture d'une transaction de cercle.
#[derive(Debug, thiserror::Error)]
pub enum ErreurAcces {
    /// L'utilisateur n'est pas membre actif de ce cercle (ou le cercle
    /// n'existe pas : les deux cas sont volontairement indiscernables).
    #[error("accès refusé à ce cercle")]
    NonMembre,
    #[error(transparent)]
    Donnees(#[from] ErreurDonnees),
}

impl From<sqlx::Error> for ErreurAcces {
    fn from(erreur: sqlx::Error) -> Self {
        Self::Donnees(ErreurDonnees::Bdd(erreur))
    }
}
