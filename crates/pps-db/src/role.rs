//! Contrôle du rôle PostgreSQL utilisé par un service (décision 0007).

use sqlx::PgExecutor;

/// Le rôle de connexion ne doit jamais pouvoir contourner la Row-Level
/// Security : un service refuse de démarrer dans ce cas.
#[derive(Debug, thiserror::Error)]
pub enum ErreurRole {
    #[error("le rôle {0} est superutilisateur : connexion refusée pour un service")]
    Superutilisateur(String),
    #[error("le rôle {0} a l'attribut BYPASSRLS : connexion refusée pour un service")]
    ContourneRls(String),
    #[error("impossible de vérifier le rôle de connexion")]
    Bdd(#[from] sqlx::Error),
}

/// Vérifie que le rôle courant n'est ni superutilisateur ni `BYPASSRLS`.
pub async fn verifier_role_applicatif<'e>(
    executeur: impl PgExecutor<'e>,
) -> Result<(), ErreurRole> {
    let role = sqlx::query!(
        r#"
        SELECT
            r.rolname::text AS "nom!",
            r.rolsuper AS "superutilisateur!",
            r.rolbypassrls AS "contourne_rls!"
        FROM pg_roles r
        WHERE r.rolname = current_user
        "#
    )
    .fetch_one(executeur)
    .await?;

    if role.superutilisateur {
        return Err(ErreurRole::Superutilisateur(role.nom));
    }
    if role.contourne_rls {
        return Err(ErreurRole::ContourneRls(role.nom));
    }
    Ok(())
}
