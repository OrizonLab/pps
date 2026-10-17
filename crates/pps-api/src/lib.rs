//! Service API de PPS.
//!
//! Au stade du socle, il expose uniquement des points de santé ; les routes
//! métier arriveront avec l'authentification (voir la feuille de route).

mod config;
mod journal;
mod routeur;
mod sante;
mod serveur;
mod verification;

pub use config::{ConfigApi, ErreurConfig, adresse_ecoute};
pub use journal::initialiser as initialiser_journal;
pub use routeur::{EtatApp, construire_routeur};
pub use serveur::{ErreurDemarrage, migrer, servir};
pub use verification::verifier_sante;
