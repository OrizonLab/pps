//! Membres des cercles.

use jiff::Timestamp;
use jiff_sqlx::Timestamp as Horodatage;
use pps_domaine::cercle::TexteCourt;
use pps_domaine::ids::{IdMembre, IdUtilisateur};
use pps_domaine::membre::{Lien, RoleMembre};
use uuid::Uuid;

use crate::contexte::TxCercle;
use crate::erreur::ErreurDonnees;

/// Un membre tel que relu en base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembreLu {
    pub id: IdMembre,
    pub utilisateur: IdUtilisateur,
    pub role: RoleMembre,
    pub lien: Lien,
    pub nom_affiche: TexteCourt,
    pub actif: bool,
}

/// Un membre à ajouter.
#[derive(Debug, Clone)]
pub struct NouveauMembre {
    pub utilisateur: IdUtilisateur,
    pub role: RoleMembre,
    pub lien: Lien,
    pub nom_affiche: TexteCourt,
}

/// Ajoute un membre au cercle de la transaction.
pub async fn inserer(
    tx: &mut TxCercle,
    nouveau: &NouveauMembre,
    maintenant: Timestamp,
) -> Result<IdMembre, ErreurDonnees> {
    let id = IdMembre::nouveau();
    let cercle = tx.cercle().uuid();
    sqlx::query!(
        r#"
        INSERT INTO app.membre (id, cercle_id, utilisateur_id, role, lien, nom_affiche, ajoute_le)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        id.uuid(),
        cercle,
        nouveau.utilisateur.uuid(),
        nouveau.role.code(),
        nouveau.lien.code(),
        nouveau.nom_affiche.as_str(),
        Horodatage::from(maintenant)
    )
    .execute(tx.connexion())
    .await?;
    Ok(id)
}

/// L'appartenance active d'un utilisateur au cercle de la transaction.
pub async fn actif_par_utilisateur(
    tx: &mut TxCercle,
    utilisateur: IdUtilisateur,
) -> Result<Option<MembreLu>, ErreurDonnees> {
    let ligne = sqlx::query!(
        r#"
        SELECT id, utilisateur_id, role, lien, nom_affiche, retire_le IS NULL AS "actif!"
        FROM app.membre
        WHERE cercle_id = $1 AND utilisateur_id = $2 AND retire_le IS NULL
        "#,
        tx.cercle().uuid(),
        utilisateur.uuid()
    )
    .fetch_optional(tx.connexion())
    .await?;
    ligne
        .map(|l| {
            vers_membre(
                l.id,
                l.utilisateur_id,
                &l.role,
                &l.lien,
                &l.nom_affiche,
                l.actif,
            )
        })
        .transpose()
}

/// Un membre (actif ou non) du cercle de la transaction.
pub async fn lire(tx: &mut TxCercle, id: IdMembre) -> Result<Option<MembreLu>, ErreurDonnees> {
    let ligne = sqlx::query!(
        r#"
        SELECT id, utilisateur_id, role, lien, nom_affiche, retire_le IS NULL AS "actif!"
        FROM app.membre
        WHERE cercle_id = $1 AND id = $2
        "#,
        tx.cercle().uuid(),
        id.uuid()
    )
    .fetch_optional(tx.connexion())
    .await?;
    ligne
        .map(|l| {
            vers_membre(
                l.id,
                l.utilisateur_id,
                &l.role,
                &l.lien,
                &l.nom_affiche,
                l.actif,
            )
        })
        .transpose()
}

/// Les membres actifs du cercle de la transaction, dans l'ordre d'arrivée.
pub async fn lister_actifs(tx: &mut TxCercle) -> Result<Vec<MembreLu>, ErreurDonnees> {
    let lignes = sqlx::query!(
        r#"
        SELECT id, utilisateur_id, role, lien, nom_affiche, retire_le IS NULL AS "actif!"
        FROM app.membre
        WHERE cercle_id = $1 AND retire_le IS NULL
        ORDER BY ajoute_le, id
        "#,
        tx.cercle().uuid()
    )
    .fetch_all(tx.connexion())
    .await?;
    lignes
        .into_iter()
        .map(|l| {
            vers_membre(
                l.id,
                l.utilisateur_id,
                &l.role,
                &l.lien,
                &l.nom_affiche,
                l.actif,
            )
        })
        .collect()
}

/// Retire un membre actif du cercle. Renvoie `false` s'il n'était pas actif.
pub async fn retirer(
    tx: &mut TxCercle,
    id: IdMembre,
    maintenant: Timestamp,
) -> Result<bool, ErreurDonnees> {
    let resultat = sqlx::query!(
        "UPDATE app.membre SET retire_le = $3 WHERE cercle_id = $1 AND id = $2 AND retire_le IS NULL",
        tx.cercle().uuid(),
        id.uuid(),
        Horodatage::from(maintenant)
    )
    .execute(tx.connexion())
    .await?;
    Ok(resultat.rows_affected() == 1)
}

fn vers_membre(
    id: Uuid,
    utilisateur: Uuid,
    role: &str,
    lien: &str,
    nom_affiche: &str,
    actif: bool,
) -> Result<MembreLu, ErreurDonnees> {
    Ok(MembreLu {
        id: IdMembre::depuis_uuid(id),
        utilisateur: IdUtilisateur::depuis_uuid(utilisateur),
        role: RoleMembre::depuis_code(role)?,
        lien: Lien::depuis_code(lien)?,
        nom_affiche: TexteCourt::nouveau(nom_affiche)?,
        actif,
    })
}
