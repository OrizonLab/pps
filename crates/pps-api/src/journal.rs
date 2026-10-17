//! Journalisation structurée.
//!
//! Rappel (CLAUDE.md) : aucune donnée de santé ni aucun secret dans les
//! journaux. Le filtre se règle avec `RUST_LOG` (par défaut `info`).

use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// `texte` pour un format lisible en développement ; JSON sinon.
pub(crate) const VAR_FORMAT: &str = "PPS_FORMAT_JOURNAL";

/// Installe le collecteur de journaux global. Sans effet s'il est déjà
/// installé.
pub fn initialiser() {
    let filtre = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=info"));
    let texte = std::env::var(VAR_FORMAT).is_ok_and(|v| v == "texte");
    let registre = tracing_subscriber::registry().with(filtre);
    let resultat = if texte {
        registre.with(tracing_subscriber::fmt::layer()).try_init()
    } else {
        registre
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(true),
            )
            .try_init()
    };
    // Déjà installé (tests) : on garde le collecteur existant.
    drop(resultat);
}
