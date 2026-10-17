//! Horodatage et horloge.

use jiff::Timestamp;

/// Source de l'heure courante, injectable pour des tests déterministes
/// (décision 0005).
pub trait Horloge: Send + Sync {
    fn maintenant(&self) -> Timestamp;
}

/// Horloge du système.
#[derive(Debug, Clone, Copy, Default)]
pub struct HorlogeSysteme;

impl Horloge for HorlogeSysteme {
    fn maintenant(&self) -> Timestamp {
        Timestamp::now()
    }
}

/// Tronque un instant à la microseconde, précision de `timestamptz` dans
/// PostgreSQL : une valeur relue en base est alors identique à celle écrite
/// (indispensable pour recalculer les empreintes du journal d'audit).
pub fn a_la_microseconde(instant: Timestamp) -> Timestamp {
    let micros = instant.as_microsecond();
    // Toujours représentable : on part d'un instant valide et on tronque.
    Timestamp::from_microsecond(micros).unwrap_or(instant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tronque_les_nanosecondes() {
        let instant = Timestamp::new(1_760_000_000, 123_456_789).unwrap();
        let tronque = a_la_microseconde(instant);
        assert_eq!(tronque.subsec_nanosecond(), 123_456_000);
        assert_eq!(a_la_microseconde(tronque), tronque);
    }
}
