//! Vérification de santé utilisée par le `HEALTHCHECK` du conteneur.
//!
//! L'image d'exécution (distroless) n'a ni shell ni `curl` : le binaire
//! interroge lui-même `/sante/vivant` en local.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const DELAI: Duration = Duration::from_secs(3);
const REQUETE: &[u8] =
    b"GET /sante/vivant HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";

/// Renvoie `true` si le service répond 200 sur `/sante/vivant`.
pub async fn verifier_sante(adresse_ecoute: SocketAddr) -> bool {
    let cible = adresse_locale(adresse_ecoute);
    let interrogation = async {
        let mut flux = TcpStream::connect(cible).await?;
        flux.write_all(REQUETE).await?;
        let mut debut = Vec::with_capacity(64);
        flux.take(64).read_to_end(&mut debut).await?;
        Ok::<_, std::io::Error>(statut_ok(&debut))
    };
    matches!(
        tokio::time::timeout(DELAI, interrogation).await,
        Ok(Ok(true))
    )
}

/// Une écoute sur toutes les interfaces s'interroge par la boucle locale.
fn adresse_locale(adresse: SocketAddr) -> SocketAddr {
    match adresse.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), adresse.port())
        }
        IpAddr::V6(ip) if ip.is_unspecified() => {
            SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), adresse.port())
        }
        _ => adresse,
    }
}

fn statut_ok(debut_reponse: &[u8]) -> bool {
    debut_reponse.starts_with(b"HTTP/1.1 200 ") || debut_reponse.starts_with(b"HTTP/1.0 200 ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnait_un_statut_200() {
        assert!(statut_ok(
            b"HTTP/1.1 200 OK\r\ncontent-type: application/json"
        ));
        assert!(!statut_ok(b"HTTP/1.1 503 Service Unavailable"));
        assert!(!statut_ok(b"HTTP/1.1 2000"));
        assert!(!statut_ok(b""));
    }

    #[test]
    fn interroge_la_boucle_locale_si_ecoute_sur_toutes_les_interfaces() {
        assert_eq!(
            adresse_locale("0.0.0.0:8080".parse().unwrap()),
            "127.0.0.1:8080".parse().unwrap()
        );
        assert_eq!(
            adresse_locale("[::]:8080".parse().unwrap()),
            "[::1]:8080".parse().unwrap()
        );
        assert_eq!(
            adresse_locale("10.0.0.5:8080".parse().unwrap()),
            "10.0.0.5:8080".parse().unwrap()
        );
    }
}
