//! Contrôle du rôle PostgreSQL utilisé par un service (décision 0007).

use sqlx::PgExecutor;

/// Le rôle de connexion d'un service ne doit jamais pouvoir contourner la
/// Row-Level Security, ni directement ni par héritage : un service refuse de
/// démarrer dans ce cas.
#[derive(Debug, thiserror::Error)]
pub enum ErreurRole {
    #[error("le rôle {0} est superutilisateur : connexion refusée pour un service")]
    Superutilisateur(String),
    #[error("le rôle {0} a l'attribut BYPASSRLS : connexion refusée pour un service")]
    ContourneRls(String),
    #[error(
        "le rôle {0} hérite d'un rôle superutilisateur ou BYPASSRLS : connexion refusée pour un service"
    )]
    HeritePrivileges(String),
    #[error(
        "le rôle {0} est membre du propriétaire des données (pps_proprietaire) : connexion refusée pour un service"
    )]
    ProprietaireDesDonnees(String),
    #[error("impossible de vérifier le rôle de connexion")]
    Bdd(#[from] sqlx::Error),
}

/// Vérifie que le rôle courant n'est ni superutilisateur, ni `BYPASSRLS`,
/// n'hérite d'aucun rôle qui l'est, et n'est pas membre du propriétaire des
/// tables (qui pourrait modifier les politiques ou l'historique).
pub async fn verifier_role_applicatif<'e>(
    executeur: impl PgExecutor<'e>,
) -> Result<(), ErreurRole> {
    let role = sqlx::query!(
        r#"
        SELECT
            r.rolname::text AS "nom!",
            r.rolsuper AS "superutilisateur!",
            r.rolbypassrls AS "contourne_rls!",
            EXISTS (
                SELECT 1 FROM pg_roles a
                WHERE a.oid <> r.oid
                  AND pg_has_role(r.oid, a.oid, 'MEMBER')
                  AND (a.rolsuper OR a.rolbypassrls)
            ) AS "herite_privileges!",
            EXISTS (
                SELECT 1 FROM pg_roles p
                WHERE p.rolname = 'pps_proprietaire'
                  AND pg_has_role(r.oid, p.oid, 'MEMBER')
            ) AS "membre_proprietaire!"
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
    if role.herite_privileges {
        return Err(ErreurRole::HeritePrivileges(role.nom));
    }
    if role.membre_proprietaire {
        return Err(ErreurRole::ProprietaireDesDonnees(role.nom));
    }
    Ok(())
}
