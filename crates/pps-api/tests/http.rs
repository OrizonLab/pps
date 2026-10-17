//! Tests HTTP du service (PostgreSQL réel, `DATABASE_URL` requis).

// Fichier de tests : les fonctions utilitaires hors `#[test]` peuvent aussi
// échouer franchement (clippy.toml ne couvre que les fonctions de test).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use pps_api::{EtatApp, construire_routeur, verifier_sante};
use sqlx::PgPool;
use tower::ServiceExt;

async fn appeler(
    bdd: PgPool,
    requete: Request<Body>,
) -> (StatusCode, axum::http::HeaderMap, serde_json::Value) {
    let reponse = construire_routeur(EtatApp::new(bdd))
        .oneshot(requete)
        .await
        .unwrap();
    let statut = reponse.status();
    let en_tetes = reponse.headers().clone();
    let corps = reponse.into_body().collect().await.unwrap().to_bytes();
    (statut, en_tetes, serde_json::from_slice(&corps).unwrap())
}

fn get(chemin: &str) -> Request<Body> {
    Request::get(chemin).body(Body::empty()).unwrap()
}

#[sqlx::test(migrator = "pps_db::MIGRATIONS")]
async fn vivant_repond_avec_les_en_tetes_de_securite(bdd: PgPool) {
    let (statut, en_tetes, corps) = appeler(bdd, get("/sante/vivant")).await;

    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps, serde_json::json!({ "statut": "vivant" }));
    assert_eq!(en_tetes[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(en_tetes[header::X_FRAME_OPTIONS], "DENY");
    assert_eq!(en_tetes[header::REFERRER_POLICY], "no-referrer");
    assert_eq!(en_tetes[header::CACHE_CONTROL], "no-store");
    assert!(
        en_tetes[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap()
            .contains("default-src 'none'")
    );
    let id = en_tetes["x-request-id"].to_str().unwrap();
    assert_eq!(uuid::Uuid::parse_str(id).unwrap().get_version_num(), 7);
}

#[sqlx::test(migrator = "pps_db::MIGRATIONS")]
async fn l_identifiant_de_requete_recu_est_propage(bdd: PgPool) {
    let requete = Request::get("/sante/vivant")
        .header("x-request-id", "id-amont-123")
        .body(Body::empty())
        .unwrap();
    let (_, en_tetes, _) = appeler(bdd, requete).await;
    assert_eq!(en_tetes["x-request-id"], "id-amont-123");
}

#[sqlx::test(migrator = "pps_db::MIGRATIONS")]
async fn pret_quand_la_base_repond(bdd: PgPool) {
    let (statut, _, corps) = appeler(bdd, get("/sante/pret")).await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(corps, serde_json::json!({ "statut": "pret" }));
}

#[sqlx::test(migrator = "pps_db::MIGRATIONS")]
async fn indisponible_quand_la_base_ne_repond_plus(bdd: PgPool) {
    bdd.close().await;
    let (statut, en_tetes, corps) = appeler(bdd, get("/sante/pret")).await;
    assert_eq!(statut, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(corps, serde_json::json!({ "statut": "indisponible" }));
    // Les en-têtes de sécurité s'appliquent aussi aux erreurs.
    assert_eq!(en_tetes[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
}

#[sqlx::test(migrator = "pps_db::MIGRATIONS")]
async fn route_inconnue_renvoie_404_sans_detail(bdd: PgPool) {
    let (statut, _, corps) = appeler(bdd, get("/admin/secret?jeton=abc")).await;
    assert_eq!(statut, StatusCode::NOT_FOUND);
    assert_eq!(corps, serde_json::json!({ "erreur": "introuvable" }));
}

#[sqlx::test(migrator = "pps_db::MIGRATIONS")]
async fn la_verification_de_sante_interroge_le_service(bdd: PgPool) {
    let ecouteur = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let adresse = ecouteur.local_addr().unwrap();
    let serveur = tokio::spawn(async move {
        axum::serve(ecouteur, construire_routeur(EtatApp::new(bdd)))
            .await
            .unwrap();
    });

    assert!(verifier_sante(adresse).await);
    serveur.abort();
}

#[tokio::test]
async fn la_verification_de_sante_echoue_sans_service() {
    // Port réservé puis libéré : plus personne n'y écoute.
    let adresse = {
        let ecouteur = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        ecouteur.local_addr().unwrap()
    };
    assert!(!verifier_sante(adresse).await);
}
