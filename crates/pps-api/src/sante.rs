//! Points de santé : `/sante/vivant` (le processus répond) et `/sante/pret`
//! (la base de données est joignable). Ils ne révèlent aucun détail interne.

use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use sqlx::{Connection, PgPool};

use crate::routeur::EtatApp;

const DELAI_VERIFICATION_BDD: Duration = Duration::from_secs(2);

#[derive(Debug, Serialize)]
struct Statut {
    statut: &'static str,
}

pub(crate) fn routes() -> Router<EtatApp> {
    Router::new()
        .route("/sante/vivant", get(vivant))
        .route("/sante/pret", get(pret))
}

async fn vivant() -> Json<Statut> {
    Json(Statut { statut: "vivant" })
}

async fn pret(State(etat): State<EtatApp>) -> (StatusCode, Json<Statut>) {
    match tokio::time::timeout(DELAI_VERIFICATION_BDD, joindre_bdd(&etat.bdd)).await {
        Ok(Ok(())) => (StatusCode::OK, Json(Statut { statut: "pret" })),
        Ok(Err(erreur)) => {
            tracing::warn!(%erreur, "base de données injoignable");
            indisponible()
        }
        Err(_) => {
            tracing::warn!("délai dépassé pour joindre la base de données");
            indisponible()
        }
    }
}

fn indisponible() -> (StatusCode, Json<Statut>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(Statut {
            statut: "indisponible",
        }),
    )
}

async fn joindre_bdd(bdd: &PgPool) -> Result<(), sqlx::Error> {
    let mut connexion = bdd.acquire().await?;
    connexion.ping().await
}
