//! Cercles de soin.

use jiff::Timestamp;
use jiff_sqlx::Timestamp as Horodatage;
use pps_domaine::cercle::{FuseauHoraire, IdentiteSenior, StatutCercle, TexteCourt};

use crate::contexte::TxCercle;
use crate::erreur::ErreurDonnees;

/// Un cercle tel que relu en base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CercleLu {
    pub statut: StatutCercle,
    pub senior: IdentiteSenior,
    pub fuseau: FuseauHoraire,
}

/// Enregistre le cercle de la transaction, en préparation.
pub async fn inserer(
    tx: &mut TxCercle,
    senior: &IdentiteSenior,
    fuseau: &FuseauHoraire,
    maintenant: Timestamp,
) -> Result<(), ErreurDonnees> {
    let id = tx.cercle().uuid();
    sqlx::query!(
        r#"
        INSERT INTO app.cercle
            (id, statut, senior_prenom, senior_appellation, senior_vouvoiement,
             fuseau_horaire, cree_le, modifie_le)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        "#,
        id,
        StatutCercle::EnPreparation.code(),
        senior.prenom.as_str(),
        senior.appellation.as_str(),
        senior.vouvoiement,
        fuseau.as_str(),
        Horodatage::from(maintenant)
    )
    .execute(tx.connexion())
    .await?;
    Ok(())
}

/// Lit le cercle de la transaction et le verrouille jusqu'à la fin de la
/// transaction (les changements de statut et l'audit sont ainsi
/// sérialisés).
pub async fn lire_pour_mise_a_jour(tx: &mut TxCercle) -> Result<CercleLu, ErreurDonnees> {
    let id = tx.cercle().uuid();
    let ligne = sqlx::query!(
        r#"
        SELECT statut, senior_prenom, senior_appellation, senior_vouvoiement, fuseau_horaire
        FROM app.cercle
        WHERE id = $1
        FOR UPDATE
        "#,
        id
    )
    .fetch_one(tx.connexion())
    .await?;
    Ok(CercleLu {
        statut: StatutCercle::depuis_code(&ligne.statut)?,
        senior: IdentiteSenior {
            prenom: TexteCourt::nouveau(&ligne.senior_prenom)?,
            appellation: TexteCourt::nouveau(&ligne.senior_appellation)?,
            vouvoiement: ligne.senior_vouvoiement,
        },
        fuseau: FuseauHoraire::nouveau(&ligne.fuseau_horaire)?,
    })
}

/// Change le statut du cercle de la transaction.
pub async fn changer_statut(
    tx: &mut TxCercle,
    statut: StatutCercle,
    maintenant: Timestamp,
) -> Result<(), ErreurDonnees> {
    let id = tx.cercle().uuid();
    sqlx::query!(
        "UPDATE app.cercle SET statut = $2, modifie_le = $3 WHERE id = $1",
        id,
        statut.code(),
        Horodatage::from(maintenant)
    )
    .execute(tx.connexion())
    .await?;
    Ok(())
}
