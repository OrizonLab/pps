//! Démarrage du service et exécution des migrations.

use pps_db::{ConfigBdd, ErreurRole, MIGRATIONS};
use tokio::net::TcpListener;

use crate::config::ConfigApi;
use crate::routeur::{EtatApp, construire_routeur};

/// Erreur fatale au démarrage. Les messages ne contiennent aucun secret.
#[derive(Debug, thiserror::Error)]
pub enum ErreurDemarrage {
    #[error("connexion à la base de données impossible")]
    Connexion(#[source] sqlx::Error),
    #[error(transparent)]
    Role(#[from] ErreurRole),
    #[error("échec des migrations")]
    Migrations(#[from] sqlx::migrate::MigrateError),
    #[error("impossible d'écouter sur l'adresse configurée")]
    Ecoute(#[source] std::io::Error),
    #[error("arrêt anormal du serveur HTTP")]
    Serveur(#[source] std::io::Error),
}

/// Démarre le service API et le laisse tourner jusqu'au signal d'arrêt.
pub async fn servir(config: ConfigApi) -> Result<(), ErreurDemarrage> {
    let bdd = pps_db::connecter(&config.bdd)
        .await
        .map_err(ErreurDemarrage::Connexion)?;
    // Décision 0007 : refus de démarrer avec un rôle qui contourne la RLS.
    pps_db::verifier_role_applicatif(&bdd).await?;

    let ecouteur = TcpListener::bind(config.adresse)
        .await
        .map_err(ErreurDemarrage::Ecoute)?;
    tracing::info!(adresse = %config.adresse, "service API démarré");

    axum::serve(ecouteur, construire_routeur(EtatApp::new(bdd.clone())))
        .with_graceful_shutdown(signal_arret())
        .await
        .map_err(ErreurDemarrage::Serveur)?;

    bdd.close().await;
    tracing::info!("service API arrêté");
    Ok(())
}

/// Applique les migrations en attente. Réservé au rôle de migration.
pub async fn migrer(config: ConfigBdd) -> Result<(), ErreurDemarrage> {
    let bdd = pps_db::connecter(&config)
        .await
        .map_err(ErreurDemarrage::Connexion)?;
    MIGRATIONS.run(&bdd).await?;
    bdd.close().await;
    tracing::info!("migrations appliquées");
    Ok(())
}

/// Attend Ctrl+C ou SIGTERM (arrêt d'un conteneur), pour terminer
/// proprement les requêtes en cours.
async fn signal_arret() {
    let interruption = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminaison = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut flux) => {
                flux.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminaison = std::future::pending::<()>();

    tokio::select! {
        () = interruption => {},
        () = terminaison => {},
    }
    tracing::info!("arrêt demandé : fin des requêtes en cours");
}
