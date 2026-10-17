//! Accès à PostgreSQL pour PPS.
//!
//! - configuration de la connexion, avec le mot de passe lu dans un fichier
//!   secret plutôt que dans une variable d'environnement ;
//! - contrôle que le rôle applicatif ne contourne pas la Row-Level Security
//!   (décision 0007) ;
//! - migrations du schéma, exécutées uniquement par la commande `migrer`.

mod config;
mod role;

pub use config::{ConfigBdd, ErreurConfigBdd};
pub use role::{ErreurRole, verifier_role_applicatif};

use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

/// Migrations du schéma, embarquées dans le binaire.
pub static MIGRATIONS: Migrator = sqlx::migrate!("../../migrations");

/// Ouvre un pool de connexions selon la configuration.
pub async fn connecter(config: &ConfigBdd) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.connexions_max())
        .acquire_timeout(config.delai_acquisition())
        .connect_with(config.options().clone())
        .await
}
