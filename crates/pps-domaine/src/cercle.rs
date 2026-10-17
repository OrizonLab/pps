//! Le cercle de soin : un senior et ses proches.

use std::fmt;

use jiff::tz::TimeZone;

/// Erreur de validation d'une valeur métier.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurValeur {
    #[error("texte vide")]
    TexteVide,
    #[error("texte trop long (au plus {max} caractères)")]
    TexteTropLong { max: usize },
    #[error("caractère de contrôle interdit")]
    CaractereInterdit,
    #[error("fuseau horaire inconnu")]
    FuseauInconnu,
    #[error("code inconnu pour {type_valeur}")]
    CodeInconnu { type_valeur: &'static str },
}

/// Cycle de vie d'un cercle.
///
/// Un cercle est créé **en préparation** par un proche : aucune donnée de
/// santé n'y est acceptée. Il devient **actif** quand le senior donne son
/// accord lors de l'appel de bienvenue, ou **refusé** s'il refuse (ses
/// données sont alors supprimées). Un cercle actif peut être **clos**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatutCercle {
    EnPreparation,
    Actif,
    Refuse,
    Clos,
}

/// Transition de statut interdite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("transition de statut interdite : {depuis:?} vers {vers:?}")]
pub struct TransitionInterdite {
    pub depuis: StatutCercle,
    pub vers: StatutCercle,
}

impl StatutCercle {
    pub fn code(self) -> &'static str {
        match self {
            Self::EnPreparation => "en_preparation",
            Self::Actif => "actif",
            Self::Refuse => "refuse",
            Self::Clos => "clos",
        }
    }

    pub fn depuis_code(code: &str) -> Result<Self, ErreurValeur> {
        match code {
            "en_preparation" => Ok(Self::EnPreparation),
            "actif" => Ok(Self::Actif),
            "refuse" => Ok(Self::Refuse),
            "clos" => Ok(Self::Clos),
            _ => Err(ErreurValeur::CodeInconnu {
                type_valeur: "statut de cercle",
            }),
        }
    }

    /// Le senior a donné son accord lors de l'appel de bienvenue.
    pub fn activer(self) -> Result<Self, TransitionInterdite> {
        self.vers(Self::Actif, self == Self::EnPreparation)
    }

    /// Le senior a refusé : rien n'est activé.
    pub fn refuser(self) -> Result<Self, TransitionInterdite> {
        self.vers(Self::Refuse, self == Self::EnPreparation)
    }

    pub fn clore(self) -> Result<Self, TransitionInterdite> {
        self.vers(Self::Clos, self == Self::Actif)
    }

    /// Seul un cercle actif peut recevoir des données de santé et des
    /// consentements.
    pub fn accepte_les_donnees_de_sante(self) -> bool {
        self == Self::Actif
    }

    fn vers(self, vers: Self, autorisee: bool) -> Result<Self, TransitionInterdite> {
        if autorisee {
            Ok(vers)
        } else {
            Err(TransitionInterdite { depuis: self, vers })
        }
    }
}

/// Texte court validé : prénom, appellation, nom affiché.
///
/// 1 à 100 caractères après suppression des espaces de bord, sans caractère
/// de contrôle. L'affichage de débogage masque le contenu (donnée
/// personnelle).
#[derive(Clone, PartialEq, Eq)]
pub struct TexteCourt(String);

impl TexteCourt {
    pub const LONGUEUR_MAX: usize = 100;

    pub fn nouveau(valeur: &str) -> Result<Self, ErreurValeur> {
        let valeur = valeur.trim();
        if valeur.is_empty() {
            return Err(ErreurValeur::TexteVide);
        }
        if valeur.chars().count() > Self::LONGUEUR_MAX {
            return Err(ErreurValeur::TexteTropLong {
                max: Self::LONGUEUR_MAX,
            });
        }
        if valeur.chars().any(char::is_control) {
            return Err(ErreurValeur::CaractereInterdit);
        }
        Ok(Self(valeur.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for TexteCourt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TexteCourt(<{} caractères>)", self.0.chars().count())
    }
}

/// Fuseau horaire IANA du senior (`Europe/Paris`, `America/Martinique`…).
/// Les rappels suivent l'heure locale du senior (décision 0011).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuseauHoraire(String);

impl FuseauHoraire {
    pub fn nouveau(nom: &str) -> Result<Self, ErreurValeur> {
        let nom = nom.trim();
        TimeZone::get(nom).map_err(|_| ErreurValeur::FuseauInconnu)?;
        Ok(Self(nom.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Comment s'adresser au senior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentiteSenior {
    pub prenom: TexteCourt,
    /// Par exemple « Madame Martin ».
    pub appellation: TexteCourt,
    pub vouvoiement: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_de_vie_nominal() {
        let actif = StatutCercle::EnPreparation.activer().unwrap();
        assert_eq!(actif, StatutCercle::Actif);
        assert_eq!(actif.clore().unwrap(), StatutCercle::Clos);
        assert_eq!(
            StatutCercle::EnPreparation.refuser().unwrap(),
            StatutCercle::Refuse
        );
    }

    #[test]
    fn transitions_interdites() {
        assert!(StatutCercle::Actif.activer().is_err());
        assert!(StatutCercle::Refuse.activer().is_err());
        assert!(StatutCercle::Actif.refuser().is_err());
        assert!(StatutCercle::EnPreparation.clore().is_err());
        assert!(StatutCercle::Clos.activer().is_err());
    }

    #[test]
    fn seul_un_cercle_actif_accepte_les_donnees_de_sante() {
        assert!(StatutCercle::Actif.accepte_les_donnees_de_sante());
        for statut in [
            StatutCercle::EnPreparation,
            StatutCercle::Refuse,
            StatutCercle::Clos,
        ] {
            assert!(!statut.accepte_les_donnees_de_sante());
        }
    }

    #[test]
    fn codes_aller_retour() {
        for statut in [
            StatutCercle::EnPreparation,
            StatutCercle::Actif,
            StatutCercle::Refuse,
            StatutCercle::Clos,
        ] {
            assert_eq!(StatutCercle::depuis_code(statut.code()).unwrap(), statut);
        }
        assert!(StatutCercle::depuis_code("inconnu").is_err());
    }

    #[test]
    fn texte_court_valide_et_nettoye() {
        assert_eq!(TexteCourt::nouveau("  Jeanne ").unwrap().as_str(), "Jeanne");
        assert_eq!(TexteCourt::nouveau("   "), Err(ErreurValeur::TexteVide));
        assert_eq!(
            TexteCourt::nouveau(&"é".repeat(101)),
            Err(ErreurValeur::TexteTropLong { max: 100 })
        );
        assert!(TexteCourt::nouveau(&"é".repeat(100)).is_ok());
        assert_eq!(
            TexteCourt::nouveau("Jea\u{0}nne"),
            Err(ErreurValeur::CaractereInterdit)
        );
    }

    #[test]
    fn texte_court_masque_dans_le_debogage() {
        let texte = TexteCourt::nouveau("Jeanne").unwrap();
        assert_eq!(format!("{texte:?}"), "TexteCourt(<6 caractères>)");
    }

    #[test]
    fn fuseaux_horaires() {
        assert!(FuseauHoraire::nouveau("Europe/Paris").is_ok());
        assert!(FuseauHoraire::nouveau("America/Martinique").is_ok());
        assert!(FuseauHoraire::nouveau("Indian/Reunion").is_ok());
        assert_eq!(
            FuseauHoraire::nouveau("Europe/Atlantide"),
            Err(ErreurValeur::FuseauInconnu)
        );
    }
}
