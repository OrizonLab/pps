//! Les membres d'un cercle (les proches qui ont un compte).

use crate::cercle::ErreurValeur;

/// Rôle dans le cercle. Le rôle ne donne **aucun** accès aux données de
/// santé : seuls les consentements du senior le font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleMembre {
    /// A créé le cercle, invite et retire les proches.
    Organisateur,
    Proche,
}

impl RoleMembre {
    pub fn code(self) -> &'static str {
        match self {
            Self::Organisateur => "organisateur",
            Self::Proche => "proche",
        }
    }

    pub fn depuis_code(code: &str) -> Result<Self, ErreurValeur> {
        match code {
            "organisateur" => Ok(Self::Organisateur),
            "proche" => Ok(Self::Proche),
            _ => Err(ErreurValeur::CodeInconnu {
                type_valeur: "rôle de membre",
            }),
        }
    }

    /// Inviter ou retirer des membres.
    pub fn peut_gerer_les_membres(self) -> bool {
        self == Self::Organisateur
    }
}

/// Lien avec le senior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lien {
    Enfant,
    PetitEnfant,
    Conjoint,
    Fratrie,
    Ami,
    Voisin,
    Autre,
}

impl Lien {
    const TOUS: [Self; 7] = [
        Self::Enfant,
        Self::PetitEnfant,
        Self::Conjoint,
        Self::Fratrie,
        Self::Ami,
        Self::Voisin,
        Self::Autre,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::Enfant => "enfant",
            Self::PetitEnfant => "petit_enfant",
            Self::Conjoint => "conjoint",
            Self::Fratrie => "fratrie",
            Self::Ami => "ami",
            Self::Voisin => "voisin",
            Self::Autre => "autre",
        }
    }

    pub fn depuis_code(code: &str) -> Result<Self, ErreurValeur> {
        Self::TOUS
            .into_iter()
            .find(|lien| lien.code() == code)
            .ok_or(ErreurValeur::CodeInconnu {
                type_valeur: "lien avec le senior",
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seul_l_organisateur_gere_les_membres() {
        assert!(RoleMembre::Organisateur.peut_gerer_les_membres());
        assert!(!RoleMembre::Proche.peut_gerer_les_membres());
    }

    #[test]
    fn codes_aller_retour() {
        for role in [RoleMembre::Organisateur, RoleMembre::Proche] {
            assert_eq!(RoleMembre::depuis_code(role.code()).unwrap(), role);
        }
        for lien in Lien::TOUS {
            assert_eq!(Lien::depuis_code(lien.code()).unwrap(), lien);
        }
        assert!(Lien::depuis_code("cousin_eloigne").is_err());
    }
}
