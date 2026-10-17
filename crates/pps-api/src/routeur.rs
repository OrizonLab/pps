//! Construction du routeur HTTP et des couches transverses : identifiant de
//! requête, traces sans données sensibles, délai maximal, taille maximale des
//! requêtes, en-têtes de sécurité, et réponse propre en cas de panique.

use std::any::Any;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::{HeaderName, HeaderValue, Request, StatusCode, header};
use axum::middleware::map_request;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::sensitive_headers::{
    SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer,
};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;
use uuid::Uuid;

use crate::sante;

const EN_TETE_ID_REQUETE: HeaderName = HeaderName::from_static("x-request-id");
const DELAI_MAX_REQUETE: Duration = Duration::from_secs(10);
const TAILLE_MAX_CORPS: usize = 1024 * 1024;

/// État partagé par les gestionnaires de requêtes.
#[derive(Debug, Clone)]
pub struct EtatApp {
    pub(crate) bdd: PgPool,
}

impl EtatApp {
    pub fn new(bdd: PgPool) -> Self {
        Self { bdd }
    }
}

/// Réponse d'erreur générique : jamais de détail interne.
#[derive(Debug, Serialize)]
struct CorpsErreur {
    erreur: &'static str,
}

fn reponse_erreur(statut: StatusCode, erreur: &'static str) -> Response {
    (statut, axum::Json(CorpsErreur { erreur })).into_response()
}

/// Construit le routeur complet du service.
pub fn construire_routeur(etat: EtatApp) -> Router {
    Router::new()
        .merge(sante::routes())
        .fallback(introuvable)
        .with_state(etat)
        .layer(
            ServiceBuilder::new()
                // En-têtes de sécurité, y compris sur les réponses d'erreur
                // produites par les couches internes.
                .layer(en_tete(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
                .layer(en_tete(header::REFERRER_POLICY, "no-referrer"))
                .layer(en_tete(header::X_FRAME_OPTIONS, "DENY"))
                .layer(en_tete(
                    header::CONTENT_SECURITY_POLICY,
                    "default-src 'none'; frame-ancestors 'none'",
                ))
                .layer(en_tete(
                    header::STRICT_TRANSPORT_SECURITY,
                    "max-age=63072000; includeSubDomains",
                ))
                .layer(en_tete(header::CACHE_CONTROL, "no-store"))
                // Un identifiant reçu n'est conservé que s'il s'agit d'un UUID.
                .layer(map_request(filtrer_id_requete))
                .layer(SetRequestIdLayer::new(EN_TETE_ID_REQUETE, IdRequeteV7))
                // Les en-têtes sensibles sont marqués de part et d'autre de la
                // couche de traces : à l'entrée pour les requêtes, avant la
                // remontée pour les réponses.
                .layer(SetSensitiveRequestHeadersLayer::new([
                    header::AUTHORIZATION,
                    header::COOKIE,
                ]))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(span_requete)
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(SetSensitiveResponseHeadersLayer::new([header::SET_COOKIE]))
                .layer(PropagateRequestIdLayer::new(EN_TETE_ID_REQUETE))
                .layer(CatchPanicLayer::custom(reponse_panique))
                // 503 plutôt que 408 : certains clients rejouent automatiquement
                // une requête après un 408, ce qui serait dangereux pour une
                // écriture non idempotente.
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::SERVICE_UNAVAILABLE,
                    DELAI_MAX_REQUETE,
                ))
                .layer(RequestBodyLimitLayer::new(TAILLE_MAX_CORPS)),
        )
}

fn en_tete(nom: HeaderName, valeur: &'static str) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::if_not_present(nom, HeaderValue::from_static(valeur))
}

/// Trace d'une requête : méthode, gabarit de route (jamais l'URL réelle, qui
/// peut contenir des jetons ou des identifiants) et identifiant de requête.
fn span_requete(requete: &Request<Body>) -> tracing::Span {
    let route = requete
        .extensions()
        .get::<MatchedPath>()
        .map_or("(route inconnue)", MatchedPath::as_str);
    let id_requete = requete
        .headers()
        .get(&EN_TETE_ID_REQUETE)
        .and_then(|valeur| valeur.to_str().ok())
        .unwrap_or_default();
    tracing::info_span!(
        "requete",
        methode = %requete.method(),
        route,
        id_requete,
    )
}

/// Retire un identifiant de requête reçu qui n'est pas un UUID (taille ou
/// contenu arbitraires) : un nouvel identifiant sera alors généré.
async fn filtrer_id_requete(mut requete: Request<Body>) -> Request<Body> {
    let valide = requete
        .headers()
        .get(&EN_TETE_ID_REQUETE)
        .is_none_or(|valeur| {
            valeur
                .to_str()
                .is_ok_and(|texte| Uuid::try_parse(texte).is_ok())
        });
    if !valide {
        requete.headers_mut().remove(&EN_TETE_ID_REQUETE);
    }
    requete
}

async fn introuvable() -> Response {
    reponse_erreur(StatusCode::NOT_FOUND, "introuvable")
}

/// Le contenu de la panique n'est jamais journalisé ni renvoyé : il pourrait
/// contenir des données.
fn reponse_panique(_contenu: Box<dyn Any + Send + 'static>) -> Response {
    tracing::error!("panique pendant le traitement d'une requête");
    reponse_erreur(StatusCode::INTERNAL_SERVER_ERROR, "erreur_interne")
}

/// Identifiants de requête en UUID v7 (triables dans le temps).
#[derive(Debug, Clone, Copy)]
struct IdRequeteV7;

impl MakeRequestId for IdRequeteV7 {
    fn make_request_id<B>(&mut self, _requete: &Request<B>) -> Option<RequestId> {
        HeaderValue::from_str(&Uuid::now_v7().to_string())
            .ok()
            .map(RequestId::new)
    }
}
