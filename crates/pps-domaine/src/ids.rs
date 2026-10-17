//! Identifiants typés : un identifiant de cercle ne peut pas être confondu
//! avec un identifiant de membre.

use std::fmt;

use uuid::Uuid;

macro_rules! identifiant {
    ($(#[$doc:meta])* $nom:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $nom(Uuid);

        impl $nom {
            /// Nouvel identifiant (UUID v7, triable dans le temps).
            pub fn nouveau() -> Self {
                Self(Uuid::now_v7())
            }

            /// Reprend un identifiant existant (lu en base, par exemple).
            pub fn depuis_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn uuid(self) -> Uuid {
                self.0
            }
        }

        impl fmt::Debug for $nom {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($nom), self.0)
            }
        }

        impl fmt::Display for $nom {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

identifiant!(
    /// Un cercle de soin, organisé autour d'un senior.
    IdCercle
);
identifiant!(
    /// Un compte de l'espace famille (un proche).
    IdUtilisateur
);
identifiant!(
    /// L'appartenance d'un utilisateur à un cercle.
    IdMembre
);
identifiant!(
    /// Un appel téléphonique passé ou reçu par PPS.
    IdAppel
);
identifiant!(
    /// Un événement de consentement (accord ou retrait).
    IdEvenementConsentement
);
