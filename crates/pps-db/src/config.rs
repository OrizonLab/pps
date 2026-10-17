//! Configuration de la connexion à PostgreSQL.

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use sqlx::postgres::PgConnectOptions;

/// URL de connexion, sans mot de passe de préférence
/// (ex. `postgres://pps_api@postgres:5432/pps?sslmode=verify-full`).
pub(crate) const VAR_URL: &str = "PPS_BDD_URL";
/// Chemin d'un fichier contenant le mot de passe (secret Docker).
pub(crate) const VAR_MOT_DE_PASSE_FICHIER: &str = "PPS_BDD_MOT_DE_PASSE_FICHIER";
/// Nombre maximal de connexions du pool.
pub(crate) const VAR_CONNEXIONS_MAX: &str = "PPS_BDD_CONNEXIONS_MAX";

const CONNEXIONS_MAX_PAR_DEFAUT: u32 = 10;
const DELAI_ACQUISITION: Duration = Duration::from_secs(5);

/// Configuration de la connexion à la base.
///
/// Son affichage de débogage ne montre jamais le mot de passe.
#[derive(Clone)]
pub struct ConfigBdd {
    options: PgConnectOptions,
    connexions_max: u32,
}

/// Erreur de configuration de la base. Les messages ne contiennent jamais
/// de secret.
#[derive(Debug, thiserror::Error)]
pub enum ErreurConfigBdd {
    #[error("variable d'environnement obligatoire absente : {0}")]
    VariableAbsente(&'static str),
    #[error("URL de base de données invalide dans {VAR_URL}")]
    UrlInvalide,
    #[error("valeur invalide pour {variable} : entier strictement positif attendu")]
    EntierInvalide { variable: &'static str },
    #[error("lecture impossible du fichier de mot de passe {chemin}")]
    LectureMotDePasse {
        chemin: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("le fichier de mot de passe {0} est vide")]
    MotDePasseVide(PathBuf),
}

impl ConfigBdd {
    /// Construit la configuration à partir des variables d'environnement.
    pub fn depuis_env() -> Result<Self, ErreurConfigBdd> {
        Self::depuis_variables(|nom| std::env::var(nom).ok())
    }

    /// Construit la configuration à partir d'une source de variables
    /// (injectable pour les tests).
    pub fn depuis_variables(
        lire: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, ErreurConfigBdd> {
        let url = lire(VAR_URL).ok_or(ErreurConfigBdd::VariableAbsente(VAR_URL))?;
        let mut options =
            PgConnectOptions::from_str(&url).map_err(|_| ErreurConfigBdd::UrlInvalide)?;

        if let Some(chemin) = lire(VAR_MOT_DE_PASSE_FICHIER) {
            options = options.password(&lire_mot_de_passe(Path::new(&chemin))?);
        }

        let connexions_max = match lire(VAR_CONNEXIONS_MAX) {
            None => CONNEXIONS_MAX_PAR_DEFAUT,
            Some(valeur) => valeur.trim().parse::<u32>().ok().filter(|n| *n > 0).ok_or(
                ErreurConfigBdd::EntierInvalide {
                    variable: VAR_CONNEXIONS_MAX,
                },
            )?,
        };

        Ok(Self {
            options,
            connexions_max,
        })
    }

    /// Options de connexion `SQLx`.
    pub fn options(&self) -> &PgConnectOptions {
        &self.options
    }

    /// Nombre maximal de connexions du pool.
    pub fn connexions_max(&self) -> u32 {
        self.connexions_max
    }

    /// Délai maximal pour obtenir une connexion du pool.
    pub fn delai_acquisition(&self) -> Duration {
        DELAI_ACQUISITION
    }
}

impl fmt::Debug for ConfigBdd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigBdd")
            .field("hote", &self.options.get_host())
            .field("port", &self.options.get_port())
            .field("base", &self.options.get_database())
            .field("utilisateur", &self.options.get_username())
            .field("connexions_max", &self.connexions_max)
            .finish_non_exhaustive()
    }
}

fn lire_mot_de_passe(chemin: &Path) -> Result<String, ErreurConfigBdd> {
    let contenu =
        std::fs::read_to_string(chemin).map_err(|source| ErreurConfigBdd::LectureMotDePasse {
            chemin: chemin.to_path_buf(),
            source,
        })?;
    // Un fichier secret se termine souvent par un saut de ligne.
    let mot_de_passe = contenu.trim_end_matches(['\n', '\r']);
    if mot_de_passe.is_empty() {
        return Err(ErreurConfigBdd::MotDePasseVide(chemin.to_path_buf()));
    }
    Ok(mot_de_passe.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn source(paires: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let carte: HashMap<String, String> = paires
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        move |nom| carte.get(nom).cloned()
    }

    fn fichier_temporaire(nom: &str, contenu: &str) -> PathBuf {
        let chemin = std::env::temp_dir().join(format!(
            "pps-test-{}-{}-{nom}",
            std::process::id(),
            std::thread::current()
                .name()
                .unwrap_or("t")
                .replace("::", "-")
        ));
        std::fs::write(&chemin, contenu).unwrap();
        chemin
    }

    #[test]
    fn url_obligatoire() {
        let erreur = ConfigBdd::depuis_variables(source(&[])).unwrap_err();
        assert!(matches!(erreur, ErreurConfigBdd::VariableAbsente(VAR_URL)));
    }

    #[test]
    fn url_invalide_refusee_sans_reprendre_son_contenu() {
        let erreur =
            ConfigBdd::depuis_variables(source(&[(VAR_URL, "pas-une-url-secrete")])).unwrap_err();
        assert!(matches!(erreur, ErreurConfigBdd::UrlInvalide));
        assert!(!erreur.to_string().contains("secrete"));
    }

    #[test]
    fn valeurs_par_defaut() {
        let config =
            ConfigBdd::depuis_variables(source(&[(VAR_URL, "postgres://pps_api@hote:5432/pps")]))
                .unwrap();
        assert_eq!(config.connexions_max(), CONNEXIONS_MAX_PAR_DEFAUT);
        assert_eq!(config.options().get_username(), "pps_api");
        assert_eq!(config.options().get_database(), Some("pps"));
    }

    #[test]
    fn connexions_max_doit_etre_un_entier_positif() {
        for valeur in ["0", "-3", "beaucoup"] {
            let erreur = ConfigBdd::depuis_variables(source(&[
                (VAR_URL, "postgres://pps_api@hote/pps"),
                (VAR_CONNEXIONS_MAX, valeur),
            ]))
            .unwrap_err();
            assert!(matches!(erreur, ErreurConfigBdd::EntierInvalide { .. }));
        }
    }

    #[test]
    fn mot_de_passe_lu_dans_le_fichier_et_jamais_affiche() {
        let chemin = fichier_temporaire("mdp", "SecretTresSensible\n");
        let config = ConfigBdd::depuis_variables(source(&[
            (VAR_URL, "postgres://pps_api@hote/pps"),
            (VAR_MOT_DE_PASSE_FICHIER, chemin.to_str().unwrap()),
        ]))
        .unwrap();
        std::fs::remove_file(&chemin).unwrap();

        let debogage = format!("{config:?}");
        assert!(!debogage.contains("SecretTresSensible"));
        assert!(debogage.contains("pps_api"));
    }

    #[test]
    fn fichier_de_mot_de_passe_vide_refuse() {
        let chemin = fichier_temporaire("vide", "\n");
        let erreur = ConfigBdd::depuis_variables(source(&[
            (VAR_URL, "postgres://pps_api@hote/pps"),
            (VAR_MOT_DE_PASSE_FICHIER, chemin.to_str().unwrap()),
        ]))
        .unwrap_err();
        std::fs::remove_file(&chemin).unwrap();
        assert!(matches!(erreur, ErreurConfigBdd::MotDePasseVide(_)));
    }

    #[test]
    fn fichier_de_mot_de_passe_absent_refuse() {
        let erreur = ConfigBdd::depuis_variables(source(&[
            (VAR_URL, "postgres://pps_api@hote/pps"),
            (VAR_MOT_DE_PASSE_FICHIER, "/chemin/qui/n/existe/pas"),
        ]))
        .unwrap_err();
        assert!(matches!(erreur, ErreurConfigBdd::LectureMotDePasse { .. }));
    }
}
