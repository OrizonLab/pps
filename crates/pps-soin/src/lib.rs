//! Moteur de soin de PPS (étage 1, sans IA, décision 0002).
//!
//! Il détient les règles qui ne doivent jamais dépendre d'un modèle d'IA :
//! qui fait partie du cercle, ce que le senior accepte de partager, et la
//! trace de chaque action dans le journal d'audit.

mod cercle;
mod erreur;

pub use cercle::{MoteurCercle, NouveauCercle, NouveauProche, VisibiliteMembre, autorisation};
pub use erreur::ErreurSoin;
