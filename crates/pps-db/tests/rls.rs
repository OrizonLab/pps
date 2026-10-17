//! Garde-fous de la Row-Level Security (décision 0007).

// Fichier de tests : les fonctions utilitaires peuvent échouer franchement.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use pps_db::MIGRATIONS;
use sqlx::PgPool;

/// Toute table du schéma applicatif a la RLS activée, forcée, et au moins une
/// politique.
#[sqlx::test(migrator = "MIGRATIONS")]
async fn toute_table_applicative_est_protegee(bdd: PgPool) {
    let tables = sqlx::query!(
        r#"
        SELECT
            c.relname::text AS "table!",
            c.relrowsecurity AS "rls_activee!",
            c.relforcerowsecurity AS "rls_forcee!",
            (SELECT count(*) FROM pg_policies p
              WHERE p.schemaname = 'app' AND p.tablename = c.relname) AS "politiques!"
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'app' AND c.relkind IN ('r', 'p')
        ORDER BY c.relname
        "#
    )
    .fetch_all(&bdd)
    .await
    .unwrap();

    assert!(!tables.is_empty());
    for table in tables {
        assert!(table.rls_activee, "{} : RLS non activée", table.table);
        assert!(table.rls_forcee, "{} : RLS non forcée", table.table);
        assert!(table.politiques > 0, "{} : aucune politique", table.table);
    }
}

/// Sans contexte de cercle, le rôle applicatif ne voit aucune ligne.
#[sqlx::test(migrator = "MIGRATIONS")]
async fn sans_contexte_aucune_ligne_n_est_visible(bdd: PgPool) {
    // Données insérées avec un contexte, en superutilisateur de test.
    let mut tx = bdd.begin().await.unwrap();
    sqlx::query!(
        r#"
        WITH contexte AS (
            SELECT set_config('app.cercle_id', '0192f0e0-0000-7000-8000-000000000001', true)
        )
        INSERT INTO app.cercle
            (id, statut, senior_prenom, senior_appellation, senior_vouvoiement,
             fuseau_horaire, cree_le, modifie_le)
        SELECT '0192f0e0-0000-7000-8000-000000000001', 'en_preparation', 'Jeanne', 'Madame',
               true, 'Europe/Paris', now(), now()
        FROM contexte
        "#
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let mut tx = bdd.begin().await.unwrap();
    sqlx::query!("SET LOCAL ROLE pps_api")
        .execute(&mut *tx)
        .await
        .unwrap();
    let visibles = sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM app.cercle"#)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(visibles, 0);

    // Une valeur de contexte invalide fait échouer la requête.
    sqlx::query_scalar!("SELECT set_config('app.cercle_id', 'pas-un-uuid', true)")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert!(
        sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM app.cercle"#)
            .fetch_one(&mut *tx)
            .await
            .is_err()
    );
}
