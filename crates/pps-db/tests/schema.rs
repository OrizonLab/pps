//! Tests d'intégration du schéma et du contrôle de rôle (PostgreSQL réel,
//! `DATABASE_URL` requis). Chaque test reçoit une base neuve, migrée.

// Fichier de tests : les fonctions utilitaires hors `#[test]` peuvent aussi
// échouer franchement (clippy.toml ne couvre que les fonctions de test).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::path::Path;

use pps_db::{ErreurRole, MIGRATIONS, verifier_role_applicatif};
use sqlx::PgPool;

#[sqlx::test(migrator = "MIGRATIONS")]
async fn le_schema_app_appartient_au_role_proprietaire(bdd: PgPool) {
    let proprietaire = sqlx::query_scalar!(
        r#"SELECT n.nspowner::regrole::text AS "proprietaire!" FROM pg_namespace n WHERE n.nspname = 'app'"#
    )
    .fetch_one(&bdd)
    .await
    .unwrap();
    assert_eq!(proprietaire, "pps_proprietaire");
}

#[sqlx::test(migrator = "MIGRATIONS")]
async fn le_role_api_peut_utiliser_le_schema_sans_y_creer(bdd: PgPool) {
    let droits = sqlx::query!(
        r#"
        SELECT
            has_schema_privilege('pps_api', 'app', 'USAGE') AS "usage!",
            has_schema_privilege('pps_api', 'app', 'CREATE') AS "creation!"
        "#
    )
    .fetch_one(&bdd)
    .await
    .unwrap();
    assert!(droits.usage);
    assert!(!droits.creation);
}

#[sqlx::test(migrator = "MIGRATIONS")]
async fn un_superutilisateur_est_refuse(bdd: PgPool) {
    // La connexion de test est superutilisateur.
    let erreur = verifier_role_applicatif(&bdd).await.unwrap_err();
    assert!(matches!(erreur, ErreurRole::Superutilisateur(_)));
}

#[sqlx::test(migrator = "MIGRATIONS")]
async fn le_role_applicatif_est_accepte(bdd: PgPool) {
    let mut transaction = bdd.begin().await.unwrap();
    sqlx::query!("SET LOCAL ROLE pps_api")
        .execute(&mut *transaction)
        .await
        .unwrap();
    verifier_role_applicatif(&mut *transaction).await.unwrap();
    transaction.rollback().await.unwrap();
}

/// Convention de la migration 0001 : chaque migration fixe le rôle
/// propriétaire et le rétablit à la fin.
#[test]
fn chaque_migration_fixe_puis_retablit_le_role_proprietaire() {
    let dossier = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
    let mut nombre = 0;
    for entree in std::fs::read_dir(&dossier).unwrap() {
        let chemin = entree.unwrap().path();
        if chemin.extension().and_then(|e| e.to_str()) != Some("sql") {
            continue;
        }
        nombre += 1;
        let contenu = std::fs::read_to_string(&chemin).unwrap();
        let instructions: Vec<&str> = contenu
            .lines()
            .map(str::trim)
            .filter(|ligne| !ligne.is_empty() && !ligne.starts_with("--"))
            .collect();
        assert!(
            instructions.contains(&"SET LOCAL ROLE pps_proprietaire;"),
            "{} : SET LOCAL ROLE pps_proprietaire; manquant",
            chemin.display()
        );
        assert_eq!(
            instructions.last(),
            Some(&"RESET ROLE;"),
            "{} : doit se terminer par RESET ROLE;",
            chemin.display()
        );
    }
    assert!(nombre > 0, "aucune migration trouvée");
}
