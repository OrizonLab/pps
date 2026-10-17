//! Configuration du service API, lue dans l'environnement.

use std::net::SocketAddr;

use pps_db::{ConfigBdd, ErreurConfigBdd};

/// Adresse d'écoute HTTP (par défaut `0.0.0.0:8080`).
pub(crate) const VAR_ADRESSE: &str = "PPS_ADRESSE";
const ADRESSE_PAR_DEFAUT: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 8080);

/// Configuration complète du service.
#[derive(Debug, Clone)]
pub struct ConfigApi {
    pub adresse: SocketAddr,
    pub bdd: ConfigBdd,
}

/// Erreur de configuration du service.
#[derive(Debug, thiserror::Error)]
pub enum ErreurConfig {
    #[error("adresse d'écoute invalide dans {VAR_ADRESSE}")]
    AdresseInvalide,
    #[error(transparent)]
    Bdd(#[from] ErreurConfigBdd),
}

impl ConfigApi {
    /// Lit la configuration dans les variables d'environnement.
    pub fn depuis_env() -> Result<Self, ErreurConfig> {
        Self::depuis_variables(|nom| std::env::var(nom).ok())
    }

    /// Lit la configuration dans une source de variables (injectable pour les
    /// tests).
    pub fn depuis_variables(lire: impl Fn(&str) -> Option<String>) -> Result<Self, ErreurConfig> {
        Ok(Self {
            adresse: adresse_ecoute(&lire)?,
            bdd: ConfigBdd::depuis_variables(&lire)?,
        })
    }
}

/// Lit seulement l'adresse d'écoute : la vérification de santé n'a pas besoin
/// des secrets de la base.
pub fn adresse_ecoute(lire: impl Fn(&str) -> Option<String>) -> Result<SocketAddr, ErreurConfig> {
    match lire(VAR_ADRESSE) {
        None => Ok(ADRESSE_PAR_DEFAUT),
        Some(valeur) => valeur
            .trim()
            .parse()
            .map_err(|_| ErreurConfig::AdresseInvalide),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adresse_par_defaut() {
        assert_eq!(adresse_ecoute(|_| None).unwrap(), ADRESSE_PAR_DEFAUT);
    }

    #[test]
    fn adresse_lue_dans_l_environnement() {
        let adresse = adresse_ecoute(|_| Some("127.0.0.1:9000".to_owned())).unwrap();
        assert_eq!(adresse, "127.0.0.1:9000".parse().unwrap());
    }

    #[test]
    fn adresse_invalide_refusee() {
        assert!(matches!(
            adresse_ecoute(|_| Some("pas une adresse".to_owned())),
            Err(ErreurConfig::AdresseInvalide)
        ));
    }

    #[test]
    fn configuration_complete() {
        let config = ConfigApi::depuis_variables(|nom| match nom {
            "PPS_BDD_URL" => Some("postgres://pps_api@hote/pps".to_owned()),
            _ => None,
        })
        .unwrap();
        assert_eq!(config.adresse, ADRESSE_PAR_DEFAUT);
        assert_eq!(config.bdd.utilisateur(), "pps_api");
    }
}
