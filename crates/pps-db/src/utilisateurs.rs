//! Comptes de l'espace famille.

use jiff::Timestamp;
use jiff_sqlx::Timestamp as Horodatage;
use pps_domaine::ids::IdUtilisateur;
use sqlx::PgPool;

use crate::contexte::fixer_utilisateur;
use crate::erreur::ErreurDonnees;

/// Crée un compte. Les informations de connexion arriveront avec
/// l'authentification (incrément 3).
pub async fn creer(bdd: &PgPool, maintenant: Timestamp) -> Result<IdUtilisateur, ErreurDonnees> {
    let id = IdUtilisateur::nouveau();
    let mut tx = bdd.begin().await?;
    // La politique de la table n'autorise que l'utilisateur du contexte.
    fixer_utilisateur(&mut tx, id).await?;
    sqlx::query!(
        "INSERT INTO app.utilisateur (id, cree_le) VALUES ($1, $2)",
        id.uuid(),
        Horodatage::from(maintenant)
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(id)
}
