//! Historique des consentements (en insertion seule).

use jiff_sqlx::Timestamp as Horodatage;
use pps_domaine::consentement::{Categorie, EvenementConsentement, Nature};
use pps_domaine::ids::{IdAppel, IdEvenementConsentement, IdMembre};

use crate::contexte::TxCercle;
use crate::erreur::ErreurDonnees;

/// Enregistre un accord ou un retrait.
pub async fn inserer(
    tx: &mut TxCercle,
    evenement: &EvenementConsentement,
) -> Result<(), ErreurDonnees> {
    let (nature, origine, appel) = evenement.nature.codes();
    sqlx::query!(
        r#"
        INSERT INTO app.consentement_evenement
            (id, cercle_id, membre_id, categorie, nature, origine, id_appel, survenu_le)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        evenement.id.uuid(),
        tx.cercle().uuid(),
        evenement.membre.uuid(),
        evenement.categorie.code(),
        nature,
        origine,
        appel.map(IdAppel::uuid),
        Horodatage::from(evenement.survenu_le)
    )
    .execute(tx.connexion())
    .await?;
    Ok(())
}

/// Tout l'historique du cercle de la transaction, dans l'ordre chronologique.
pub async fn lister(tx: &mut TxCercle) -> Result<Vec<EvenementConsentement>, ErreurDonnees> {
    let lignes = sqlx::query!(
        r#"
        SELECT id, membre_id, categorie, nature, origine, id_appel, survenu_le
        FROM app.consentement_evenement
        WHERE cercle_id = $1
        ORDER BY survenu_le, id
        "#,
        tx.cercle().uuid()
    )
    .fetch_all(tx.connexion())
    .await?;
    lignes
        .into_iter()
        .map(|l| {
            Ok(EvenementConsentement {
                id: IdEvenementConsentement::depuis_uuid(l.id),
                membre: IdMembre::depuis_uuid(l.membre_id),
                categorie: Categorie::depuis_code(&l.categorie)?,
                nature: Nature::depuis_codes(
                    &l.nature,
                    &l.origine,
                    l.id_appel.map(IdAppel::depuis_uuid),
                )?,
                survenu_le: l.survenu_le.to_jiff(),
            })
        })
        .collect()
}
