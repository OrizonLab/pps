//! Règles métier de PPS, sans aucune dépendance technique (ni base de
//! données, ni réseau) : elles se testent seules.
//!
//! - [`cercle`] : le cercle de soin autour d'un senior et son cycle de vie ;
//! - [`membre`] : les proches membres du cercle et leurs rôles ;
//! - [`consentement`] : qui voit quoi, décidé par le senior (décision 0006 :
//!   un accord ne se recueille que lors d'un appel émis par PPS) ;
//! - [`audit`] : le journal d'audit chaîné par empreintes (décision 0007).

pub mod audit;
pub mod cercle;
pub mod consentement;
pub mod horodatage;
pub mod ids;
pub mod membre;
