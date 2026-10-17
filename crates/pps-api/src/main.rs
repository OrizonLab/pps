//! Point d'entrée du binaire `pps-api`.
//!
//! Commandes :
//! - `serveur` (par défaut) : démarre le service API ;
//! - `migrer` : applique les migrations (rôle de migration uniquement) ;
//! - `verifier-sante` : vérification locale pour le `HEALTHCHECK` Docker.

use std::error::Error;
use std::fmt::Write as _;
use std::process::ExitCode;

use pps_api::{ConfigApi, adresse_ecoute, initialiser_journal, migrer, servir, verifier_sante};
use pps_db::ConfigBdd;

#[tokio::main]
async fn main() -> ExitCode {
    let commande = std::env::args().nth(1);
    match commande.as_deref() {
        None | Some("serveur") => {
            initialiser_journal();
            let resultat = match ConfigApi::depuis_env() {
                Ok(config) => servir(config).await.map_err(Into::into),
                Err(erreur) => Err(Box::new(erreur) as Box<dyn Error>),
            };
            terminer(resultat, "le service API s'est arrêté sur une erreur")
        }
        Some("migrer") => {
            initialiser_journal();
            let resultat = match ConfigBdd::depuis_env() {
                Ok(config) => migrer(config).await.map_err(Into::into),
                Err(erreur) => Err(Box::new(erreur) as Box<dyn Error>),
            };
            terminer(resultat, "les migrations ont échoué")
        }
        Some("verifier-sante") => {
            let sain = match adresse_ecoute(|nom| std::env::var(nom).ok()) {
                Ok(adresse) => verifier_sante(adresse).await,
                Err(_) => false,
            };
            if sain {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some(_) => {
            initialiser_journal();
            tracing::error!("commande inconnue (attendu : serveur, migrer, verifier-sante)");
            ExitCode::from(2)
        }
    }
}

fn terminer(resultat: Result<(), Box<dyn Error>>, contexte: &str) -> ExitCode {
    match resultat {
        Ok(()) => ExitCode::SUCCESS,
        Err(erreur) => {
            tracing::error!(erreur = %chaine_erreurs(erreur.as_ref()), "{contexte}");
            ExitCode::FAILURE
        }
    }
}

/// Message d'erreur avec ses causes, pour un diagnostic complet.
fn chaine_erreurs(erreur: &dyn Error) -> String {
    let mut message = erreur.to_string();
    let mut cause = erreur.source();
    while let Some(c) = cause {
        let _ = write!(message, " : {c}");
        cause = c.source();
    }
    message
}
