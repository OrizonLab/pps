//! Consentements : le senior décide, catégorie par catégorie et proche par
//! proche, de ce que chacun peut voir (docs/10-suivi-sante-a-distance.md).
//!
//! L'historique des accords et des retraits est conservé ; l'état courant en
//! est déduit. Sans accord explicite, rien n'est visible.

use std::collections::{BTreeMap, BTreeSet};

use jiff::Timestamp;

use crate::cercle::{ErreurValeur, StatutCercle};
use crate::ids::{IdAppel, IdEvenementConsentement, IdMembre};

/// Catégorie de données partageable avec un proche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Categorie {
    /// La prise (ou l'oubli) des médicaments.
    PrisesMedicaments,
    /// Les rendez-vous médicaux.
    RendezVous,
    /// Humeur, sommeil, douleur, appétit, tels que le senior les exprime.
    BienEtre,
    /// La réponse (ou l'absence de réponse) à l'appel du matin.
    CheckIn,
    /// Recevoir les alertes de la cascade.
    Alertes,
}

impl Categorie {
    pub const TOUTES: [Self; 5] = [
        Self::PrisesMedicaments,
        Self::RendezVous,
        Self::BienEtre,
        Self::CheckIn,
        Self::Alertes,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::PrisesMedicaments => "prises_medicaments",
            Self::RendezVous => "rendez_vous",
            Self::BienEtre => "bien_etre",
            Self::CheckIn => "check_in",
            Self::Alertes => "alertes",
        }
    }

    pub fn depuis_code(code: &str) -> Result<Self, ErreurValeur> {
        Self::TOUTES
            .into_iter()
            .find(|categorie| categorie.code() == code)
            .ok_or(ErreurValeur::CodeInconnu {
                type_valeur: "catégorie de consentement",
            })
    }
}

/// D'où vient un accord. Un accord ne se recueille **que** lors d'un appel
/// émis par PPS vers un numéro vérifié, car le numéro d'un appelant peut être
/// usurpé (décision 0006). Aucune autre origine n'est représentable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrigineAccord {
    AppelSortant(IdAppel),
}

/// D'où vient un retrait. Un retrait est accepté par tout canal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrigineRetrait {
    AppelSortant(IdAppel),
    AppelEntrant(IdAppel),
    /// Demande transmise par un conseiller PPS.
    Conseiller,
    /// Le membre renonce lui-même à son accès.
    RenonciationDuMembre,
}

/// Accord ou retrait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nature {
    Accord(OrigineAccord),
    Retrait(OrigineRetrait),
}

impl Nature {
    /// Codes de persistance : (nature, origine, appel éventuel).
    pub fn codes(self) -> (&'static str, &'static str, Option<IdAppel>) {
        match self {
            Self::Accord(OrigineAccord::AppelSortant(appel)) => {
                ("accord", "appel_sortant", Some(appel))
            }
            Self::Retrait(OrigineRetrait::AppelSortant(appel)) => {
                ("retrait", "appel_sortant", Some(appel))
            }
            Self::Retrait(OrigineRetrait::AppelEntrant(appel)) => {
                ("retrait", "appel_entrant", Some(appel))
            }
            Self::Retrait(OrigineRetrait::Conseiller) => ("retrait", "conseiller", None),
            Self::Retrait(OrigineRetrait::RenonciationDuMembre) => {
                ("retrait", "renonciation_membre", None)
            }
        }
    }

    pub fn depuis_codes(
        nature: &str,
        origine: &str,
        appel: Option<IdAppel>,
    ) -> Result<Self, ErreurValeur> {
        let inconnu = ErreurValeur::CodeInconnu {
            type_valeur: "événement de consentement",
        };
        match (nature, origine, appel) {
            ("accord", "appel_sortant", Some(appel)) => {
                Ok(Self::Accord(OrigineAccord::AppelSortant(appel)))
            }
            ("retrait", "appel_sortant", Some(appel)) => {
                Ok(Self::Retrait(OrigineRetrait::AppelSortant(appel)))
            }
            ("retrait", "appel_entrant", Some(appel)) => {
                Ok(Self::Retrait(OrigineRetrait::AppelEntrant(appel)))
            }
            ("retrait", "conseiller", None) => Ok(Self::Retrait(OrigineRetrait::Conseiller)),
            ("retrait", "renonciation_membre", None) => {
                Ok(Self::Retrait(OrigineRetrait::RenonciationDuMembre))
            }
            _ => Err(inconnu),
        }
    }
}

/// Un accord ou un retrait, tel qu'enregistré.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvenementConsentement {
    pub id: IdEvenementConsentement,
    pub membre: IdMembre,
    pub categorie: Categorie,
    pub nature: Nature,
    pub survenu_le: Timestamp,
}

/// Qui demande un retrait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuteurRetrait {
    Senior,
    /// Le membre concerné, pour son propre accès uniquement.
    LeMembreLuiMeme,
}

/// Refus d'enregistrer un accord ou un retrait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RefusConsentement {
    #[error("le cercle n'est pas actif : aucun accord ne peut être enregistré")]
    CercleNonActif,
    #[error("ce membre ne fait plus partie du cercle")]
    MembreInactif,
    #[error("cette origine de retrait ne correspond pas à son auteur")]
    OrigineIncoherente,
}

/// Un accord n'est possible que pour un membre actif d'un cercle actif.
pub fn verifier_accord(statut: StatutCercle, membre_actif: bool) -> Result<(), RefusConsentement> {
    if !statut.accepte_les_donnees_de_sante() {
        return Err(RefusConsentement::CercleNonActif);
    }
    if !membre_actif {
        return Err(RefusConsentement::MembreInactif);
    }
    Ok(())
}

/// Le senior retire par tout canal ; un membre ne peut que renoncer à son
/// propre accès.
pub fn verifier_retrait(
    auteur: AuteurRetrait,
    origine: OrigineRetrait,
) -> Result<(), RefusConsentement> {
    let coherent = match auteur {
        AuteurRetrait::Senior => origine != OrigineRetrait::RenonciationDuMembre,
        AuteurRetrait::LeMembreLuiMeme => origine == OrigineRetrait::RenonciationDuMembre,
    };
    if coherent {
        Ok(())
    } else {
        Err(RefusConsentement::OrigineIncoherente)
    }
}

/// État courant des consentements d'un cercle, déduit de l'historique.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EtatConsentements {
    accordes: BTreeSet<(IdMembre, Categorie)>,
}

impl EtatConsentements {
    /// Rejoue l'historique dans l'ordre chronologique (puis par identifiant,
    /// en cas d'égalité) : le dernier événement de chaque couple membre et
    /// catégorie l'emporte.
    pub fn depuis_evenements<'a>(
        evenements: impl IntoIterator<Item = &'a EvenementConsentement>,
    ) -> Self {
        let mut tries: Vec<&EvenementConsentement> = evenements.into_iter().collect();
        tries.sort_by_key(|e| (e.survenu_le, e.id));
        let mut dernier: BTreeMap<(IdMembre, Categorie), bool> = BTreeMap::new();
        for evenement in tries {
            let accorde = matches!(evenement.nature, Nature::Accord(_));
            dernier.insert((evenement.membre, evenement.categorie), accorde);
        }
        Self {
            accordes: dernier
                .into_iter()
                .filter_map(|(cle, accorde)| accorde.then_some(cle))
                .collect(),
        }
    }

    /// Ce membre peut-il voir cette catégorie ? Non par défaut.
    pub fn autorise(&self, membre: IdMembre, categorie: Categorie) -> bool {
        self.accordes.contains(&(membre, categorie))
    }

    /// Les catégories visibles par un membre.
    pub fn categories_de(&self, membre: IdMembre) -> BTreeSet<Categorie> {
        self.accordes
            .iter()
            .filter(|(m, _)| *m == membre)
            .map(|(_, categorie)| *categorie)
            .collect()
    }

    /// Les membres qui voient une catégorie.
    pub fn membres_autorises(&self, categorie: Categorie) -> BTreeSet<IdMembre> {
        self.accordes
            .iter()
            .filter(|(_, c)| *c == categorie)
            .map(|(membre, _)| *membre)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evenement(
        membre: IdMembre,
        categorie: Categorie,
        nature: Nature,
        seconde: i64,
    ) -> EvenementConsentement {
        EvenementConsentement {
            id: IdEvenementConsentement::nouveau(),
            membre,
            categorie,
            nature,
            survenu_le: Timestamp::from_second(1_760_000_000 + seconde).unwrap(),
        }
    }

    fn accord() -> Nature {
        Nature::Accord(OrigineAccord::AppelSortant(IdAppel::nouveau()))
    }

    #[test]
    fn rien_n_est_visible_par_defaut() {
        let etat = EtatConsentements::depuis_evenements(&[]);
        let membre = IdMembre::nouveau();
        for categorie in Categorie::TOUTES {
            assert!(!etat.autorise(membre, categorie));
        }
    }

    #[test]
    fn accord_par_membre_et_par_categorie() {
        let sophie = IdMembre::nouveau();
        let lucas = IdMembre::nouveau();
        let etat = EtatConsentements::depuis_evenements(&[evenement(
            sophie,
            Categorie::PrisesMedicaments,
            accord(),
            0,
        )]);
        assert!(etat.autorise(sophie, Categorie::PrisesMedicaments));
        assert!(!etat.autorise(sophie, Categorie::BienEtre));
        assert!(!etat.autorise(lucas, Categorie::PrisesMedicaments));
        assert_eq!(
            etat.membres_autorises(Categorie::PrisesMedicaments),
            BTreeSet::from([sophie])
        );
    }

    #[test]
    fn le_dernier_evenement_l_emporte_quel_que_soit_l_ordre_recu() {
        let sophie = IdMembre::nouveau();
        let accord_initial = evenement(sophie, Categorie::BienEtre, accord(), 0);
        let retrait = evenement(
            sophie,
            Categorie::BienEtre,
            Nature::Retrait(OrigineRetrait::Conseiller),
            10,
        );
        let etat = EtatConsentements::depuis_evenements(&[retrait.clone(), accord_initial.clone()]);
        assert!(!etat.autorise(sophie, Categorie::BienEtre));

        let nouvel_accord = evenement(sophie, Categorie::BienEtre, accord(), 20);
        let etat = EtatConsentements::depuis_evenements(&[nouvel_accord, retrait, accord_initial]);
        assert!(etat.autorise(sophie, Categorie::BienEtre));
        assert_eq!(
            etat.categories_de(sophie),
            BTreeSet::from([Categorie::BienEtre])
        );
    }

    #[test]
    fn un_accord_exige_un_cercle_actif_et_un_membre_actif() {
        assert_eq!(verifier_accord(StatutCercle::Actif, true), Ok(()));
        assert_eq!(
            verifier_accord(StatutCercle::EnPreparation, true),
            Err(RefusConsentement::CercleNonActif)
        );
        assert_eq!(
            verifier_accord(StatutCercle::Clos, true),
            Err(RefusConsentement::CercleNonActif)
        );
        assert_eq!(
            verifier_accord(StatutCercle::Actif, false),
            Err(RefusConsentement::MembreInactif)
        );
    }

    #[test]
    fn qui_peut_retirer_un_accord() {
        let appel = IdAppel::nouveau();
        assert!(
            verifier_retrait(AuteurRetrait::Senior, OrigineRetrait::AppelEntrant(appel)).is_ok()
        );
        assert!(verifier_retrait(AuteurRetrait::Senior, OrigineRetrait::Conseiller).is_ok());
        assert!(
            verifier_retrait(AuteurRetrait::Senior, OrigineRetrait::RenonciationDuMembre).is_err()
        );
        assert!(
            verifier_retrait(
                AuteurRetrait::LeMembreLuiMeme,
                OrigineRetrait::RenonciationDuMembre
            )
            .is_ok()
        );
        assert!(
            verifier_retrait(AuteurRetrait::LeMembreLuiMeme, OrigineRetrait::Conseiller).is_err()
        );
    }

    #[test]
    fn codes_de_persistance_aller_retour() {
        let appel = IdAppel::nouveau();
        for nature in [
            Nature::Accord(OrigineAccord::AppelSortant(appel)),
            Nature::Retrait(OrigineRetrait::AppelSortant(appel)),
            Nature::Retrait(OrigineRetrait::AppelEntrant(appel)),
            Nature::Retrait(OrigineRetrait::Conseiller),
            Nature::Retrait(OrigineRetrait::RenonciationDuMembre),
        ] {
            let (n, o, a) = nature.codes();
            assert_eq!(Nature::depuis_codes(n, o, a).unwrap(), nature);
        }
        // Un accord sans appel émis par PPS n'est pas représentable.
        assert!(Nature::depuis_codes("accord", "conseiller", None).is_err());
        assert!(Nature::depuis_codes("accord", "appel_entrant", Some(appel)).is_err());
        assert!(Nature::depuis_codes("accord", "appel_sortant", None).is_err());
        for categorie in Categorie::TOUTES {
            assert_eq!(Categorie::depuis_code(categorie.code()).unwrap(), categorie);
        }
    }
}
