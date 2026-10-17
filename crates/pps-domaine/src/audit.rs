//! Journal d'audit chaîné (décision 0007) : chaque entrée d'un cercle
//! contient l'empreinte de la précédente, ce qui rend toute modification ou
//! suppression détectable. Le journal ne contient que des identifiants et
//! des actions, jamais de donnée de santé.

use std::fmt;

use jiff::Timestamp;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::cercle::ErreurValeur;
use crate::horodatage::a_la_microseconde;
use crate::ids::{IdCercle, IdEvenementConsentement, IdMembre, IdUtilisateur};

/// Qui a agi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acteur {
    /// Un proche connecté à l'espace famille.
    Utilisateur(IdUtilisateur),
    /// Le senior, lors d'un appel.
    Senior,
    /// Un traitement automatique du moteur de soin.
    Systeme,
}

impl Acteur {
    pub fn code(self) -> &'static str {
        match self {
            Self::Utilisateur(_) => "utilisateur",
            Self::Senior => "senior",
            Self::Systeme => "systeme",
        }
    }

    pub fn id(self) -> Option<Uuid> {
        match self {
            Self::Utilisateur(id) => Some(id.uuid()),
            Self::Senior | Self::Systeme => None,
        }
    }

    pub fn depuis_codes(code: &str, id: Option<Uuid>) -> Result<Self, ErreurValeur> {
        match (code, id) {
            ("utilisateur", Some(id)) => Ok(Self::Utilisateur(IdUtilisateur::depuis_uuid(id))),
            ("senior", None) => Ok(Self::Senior),
            ("systeme", None) => Ok(Self::Systeme),
            _ => Err(ErreurValeur::CodeInconnu {
                type_valeur: "acteur d'audit",
            }),
        }
    }
}

/// Ce qui a été fait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    CercleCree,
    CercleActive,
    CercleRefuse,
    CercleClos,
    MembreAjoute,
    MembreRetire,
    ConsentementAccorde,
    ConsentementRetire,
}

impl Action {
    const TOUTES: [Self; 8] = [
        Self::CercleCree,
        Self::CercleActive,
        Self::CercleRefuse,
        Self::CercleClos,
        Self::MembreAjoute,
        Self::MembreRetire,
        Self::ConsentementAccorde,
        Self::ConsentementRetire,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::CercleCree => "cercle_cree",
            Self::CercleActive => "cercle_active",
            Self::CercleRefuse => "cercle_refuse",
            Self::CercleClos => "cercle_clos",
            Self::MembreAjoute => "membre_ajoute",
            Self::MembreRetire => "membre_retire",
            Self::ConsentementAccorde => "consentement_accorde",
            Self::ConsentementRetire => "consentement_retire",
        }
    }

    pub fn depuis_code(code: &str) -> Result<Self, ErreurValeur> {
        Self::TOUTES
            .into_iter()
            .find(|action| action.code() == code)
            .ok_or(ErreurValeur::CodeInconnu {
                type_valeur: "action d'audit",
            })
    }
}

/// Sur quoi porte l'action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Objet {
    Cercle(IdCercle),
    Membre(IdMembre),
    Consentement(IdEvenementConsentement),
}

impl Objet {
    pub fn code(self) -> &'static str {
        match self {
            Self::Cercle(_) => "cercle",
            Self::Membre(_) => "membre",
            Self::Consentement(_) => "consentement",
        }
    }

    pub fn id(self) -> Uuid {
        match self {
            Self::Cercle(id) => id.uuid(),
            Self::Membre(id) => id.uuid(),
            Self::Consentement(id) => id.uuid(),
        }
    }

    pub fn depuis_codes(code: &str, id: Uuid) -> Result<Self, ErreurValeur> {
        match code {
            "cercle" => Ok(Self::Cercle(IdCercle::depuis_uuid(id))),
            "membre" => Ok(Self::Membre(IdMembre::depuis_uuid(id))),
            "consentement" => Ok(Self::Consentement(IdEvenementConsentement::depuis_uuid(id))),
            _ => Err(ErreurValeur::CodeInconnu {
                type_valeur: "objet d'audit",
            }),
        }
    }
}

/// Empreinte SHA-256.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Empreinte([u8; 32]);

impl Empreinte {
    /// Empreinte « précédente » de la première entrée d'un cercle.
    pub const ORIGINE: Self = Self([0; 32]);

    pub fn octets(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn depuis_octets(octets: &[u8]) -> Result<Self, ErreurValeur> {
        <[u8; 32]>::try_from(octets)
            .map(Self)
            .map_err(|_| ErreurValeur::CodeInconnu {
                type_valeur: "empreinte",
            })
    }
}

impl fmt::Debug for Empreinte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Empreinte(")?;
        for octet in &self.0 {
            write!(f, "{octet:02x}")?;
        }
        write!(f, ")")
    }
}

/// Une action à journaliser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NouvelleEntree {
    pub acteur: Acteur,
    pub action: Action,
    pub objet: Objet,
    pub survenu_le: Timestamp,
}

/// Une entrée du journal, chaînée à la précédente du même cercle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntreeAudit {
    pub cercle: IdCercle,
    pub sequence: i64,
    pub acteur: Acteur,
    pub action: Action,
    pub objet: Objet,
    pub survenu_le: Timestamp,
    pub empreinte_precedente: Empreinte,
    pub empreinte: Empreinte,
}

/// Erreur de construction ou de vérification du journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ErreurAudit {
    #[error("numéro de séquence du journal épuisé")]
    SequenceEpuisee,
    #[error("journal d'audit altéré à la séquence {sequence}")]
    RuptureChaine { sequence: i64 },
}

impl EntreeAudit {
    /// Chaîne une nouvelle entrée à la dernière entrée du cercle (`None` pour
    /// la première).
    pub fn chainer(
        cercle: IdCercle,
        precedente: Option<(i64, Empreinte)>,
        nouvelle: NouvelleEntree,
    ) -> Result<Self, ErreurAudit> {
        let (sequence, empreinte_precedente) = match precedente {
            None => (1, Empreinte::ORIGINE),
            Some((sequence, empreinte)) => (
                sequence
                    .checked_add(1)
                    .ok_or(ErreurAudit::SequenceEpuisee)?,
                empreinte,
            ),
        };
        let mut entree = Self {
            cercle,
            sequence,
            acteur: nouvelle.acteur,
            action: nouvelle.action,
            objet: nouvelle.objet,
            survenu_le: a_la_microseconde(nouvelle.survenu_le),
            empreinte_precedente,
            empreinte: Empreinte::ORIGINE,
        };
        entree.empreinte = entree.calculer_empreinte();
        Ok(entree)
    }

    /// Recalcule l'empreinte à partir du contenu de l'entrée.
    pub fn calculer_empreinte(&self) -> Empreinte {
        let mut hachage = Sha256::new();
        hachage.update(b"pps-audit-v1\0");
        hachage.update(self.cercle.uuid().as_bytes());
        hachage.update(self.sequence.to_be_bytes());
        champ(&mut hachage, self.acteur.code().as_bytes());
        hachage.update(self.acteur.id().unwrap_or(Uuid::nil()).as_bytes());
        champ(&mut hachage, self.action.code().as_bytes());
        champ(&mut hachage, self.objet.code().as_bytes());
        hachage.update(self.objet.id().as_bytes());
        hachage.update(self.survenu_le.as_microsecond().to_be_bytes());
        hachage.update(self.empreinte_precedente.octets());
        Empreinte(hachage.finalize().into())
    }
}

/// Champ de longueur variable, préfixé par sa longueur : la concaténation
/// reste sans ambiguïté.
fn champ(hachage: &mut Sha256, octets: &[u8]) {
    let longueur = u32::try_from(octets.len()).unwrap_or(u32::MAX);
    hachage.update(longueur.to_be_bytes());
    hachage.update(octets);
}

/// Vérifie le journal complet d'un cercle, trié par séquence : séquences
/// contiguës à partir de 1, chaînage et empreintes intacts.
pub fn verifier_chaine(entrees: &[EntreeAudit]) -> Result<(), ErreurAudit> {
    let mut precedente = Empreinte::ORIGINE;
    let mut attendue: i64 = 1;
    let mut cercle = None;
    for entree in entrees {
        let meme_cercle = *cercle.get_or_insert(entree.cercle) == entree.cercle;
        if !meme_cercle
            || entree.sequence != attendue
            || entree.empreinte_precedente != precedente
            || entree.calculer_empreinte() != entree.empreinte
        {
            return Err(ErreurAudit::RuptureChaine {
                sequence: entree.sequence,
            });
        }
        precedente = entree.empreinte;
        attendue = attendue
            .checked_add(1)
            .ok_or(ErreurAudit::SequenceEpuisee)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn journal(cercle: IdCercle, longueur: usize) -> Vec<EntreeAudit> {
        let mut entrees: Vec<EntreeAudit> = Vec::new();
        for i in 0..longueur {
            let precedente = entrees.last().map(|e| (e.sequence, e.empreinte));
            entrees.push(
                EntreeAudit::chainer(
                    cercle,
                    precedente,
                    NouvelleEntree {
                        acteur: Acteur::Utilisateur(IdUtilisateur::nouveau()),
                        action: Action::MembreAjoute,
                        objet: Objet::Membre(IdMembre::nouveau()),
                        survenu_le: Timestamp::new(
                            1_760_000_000 + i64::try_from(i).unwrap(),
                            987_654_321,
                        )
                        .unwrap(),
                    },
                )
                .unwrap(),
            );
        }
        entrees
    }

    #[test]
    fn un_journal_intact_est_valide() {
        let entrees = journal(IdCercle::nouveau(), 5);
        assert_eq!(entrees[0].sequence, 1);
        assert_eq!(entrees[0].empreinte_precedente, Empreinte::ORIGINE);
        assert_eq!(entrees[4].sequence, 5);
        assert_eq!(verifier_chaine(&entrees), Ok(()));
        assert_eq!(verifier_chaine(&[]), Ok(()));
    }

    #[test]
    fn l_horodatage_est_tronque_a_la_microseconde() {
        let entrees = journal(IdCercle::nouveau(), 1);
        assert_eq!(entrees[0].survenu_le.subsec_nanosecond(), 987_654_000);
    }

    #[test]
    fn une_modification_est_detectee() {
        let mut entrees = journal(IdCercle::nouveau(), 4);
        entrees[2].action = Action::ConsentementAccorde;
        assert_eq!(
            verifier_chaine(&entrees),
            Err(ErreurAudit::RuptureChaine { sequence: 3 })
        );
    }

    #[test]
    fn une_suppression_est_detectee() {
        let mut entrees = journal(IdCercle::nouveau(), 4);
        entrees.remove(1);
        assert_eq!(
            verifier_chaine(&entrees),
            Err(ErreurAudit::RuptureChaine { sequence: 3 })
        );
    }

    #[test]
    fn une_empreinte_recalculee_ne_suffit_pas_a_masquer_une_modification() {
        let mut entrees = journal(IdCercle::nouveau(), 3);
        entrees[0].action = Action::CercleClos;
        entrees[0].empreinte = entrees[0].calculer_empreinte();
        // L'entrée suivante référence toujours l'ancienne empreinte.
        assert_eq!(
            verifier_chaine(&entrees),
            Err(ErreurAudit::RuptureChaine { sequence: 2 })
        );
    }

    #[test]
    fn des_entrees_de_cercles_differents_ne_forment_pas_une_chaine() {
        let mut entrees = journal(IdCercle::nouveau(), 2);
        let autre = journal(IdCercle::nouveau(), 2);
        entrees[1] = autre[1].clone();
        assert!(verifier_chaine(&entrees).is_err());
    }

    #[test]
    fn codes_aller_retour() {
        for action in Action::TOUTES {
            assert_eq!(Action::depuis_code(action.code()).unwrap(), action);
        }
        let id = IdUtilisateur::nouveau();
        for acteur in [Acteur::Utilisateur(id), Acteur::Senior, Acteur::Systeme] {
            assert_eq!(
                Acteur::depuis_codes(acteur.code(), acteur.id()).unwrap(),
                acteur
            );
        }
        assert!(Acteur::depuis_codes("utilisateur", None).is_err());
        let objet = Objet::Cercle(IdCercle::nouveau());
        assert_eq!(
            Objet::depuis_codes(objet.code(), objet.id()).unwrap(),
            objet
        );
    }
}
