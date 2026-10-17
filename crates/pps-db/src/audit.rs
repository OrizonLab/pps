//! Journal d'audit chaîné (en insertion seule).

use jiff_sqlx::Timestamp as Horodatage;
use pps_domaine::audit::{Acteur, Action, Empreinte, EntreeAudit, NouvelleEntree, Objet};

use crate::contexte::TxCercle;
use crate::erreur::ErreurDonnees;

/// Ajoute une entrée au journal du cercle de la transaction.
///
/// Le cercle doit avoir été verrouillé dans la transaction (voir
/// [`crate::cercles::lire_pour_mise_a_jour`]) ou venir d'être créé : deux
/// écritures concurrentes ne peuvent donc pas se chaîner à la même entrée.
/// La clé primaire (cercle, séquence) l'empêcherait de toute façon.
pub async fn journaliser(
    tx: &mut TxCercle,
    nouvelle: NouvelleEntree,
) -> Result<EntreeAudit, ErreurDonnees> {
    let cercle = tx.cercle();
    let derniere = sqlx::query!(
        r#"
        SELECT sequence, empreinte
        FROM app.audit
        WHERE cercle_id = $1
        ORDER BY sequence DESC
        LIMIT 1
        "#,
        cercle.uuid()
    )
    .fetch_optional(tx.connexion())
    .await?;
    let precedente = derniere
        .map(|l| Ok::<_, ErreurDonnees>((l.sequence, Empreinte::depuis_octets(&l.empreinte)?)))
        .transpose()?;

    let entree = EntreeAudit::chainer(cercle, precedente, nouvelle)?;
    sqlx::query!(
        r#"
        INSERT INTO app.audit
            (cercle_id, sequence, acteur_type, acteur_id, action, objet_type, objet_id,
             survenu_le, empreinte_precedente, empreinte)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        cercle.uuid(),
        entree.sequence,
        entree.acteur.code(),
        entree.acteur.id(),
        entree.action.code(),
        entree.objet.code(),
        entree.objet.id(),
        Horodatage::from(entree.survenu_le),
        entree.empreinte_precedente.octets().as_slice(),
        entree.empreinte.octets().as_slice()
    )
    .execute(tx.connexion())
    .await?;
    Ok(entree)
}

/// Le journal complet du cercle de la transaction, par séquence croissante.
pub async fn lire_journal(tx: &mut TxCercle) -> Result<Vec<EntreeAudit>, ErreurDonnees> {
    let cercle = tx.cercle();
    let lignes = sqlx::query!(
        r#"
        SELECT sequence, acteur_type, acteur_id, action, objet_type, objet_id,
               survenu_le, empreinte_precedente, empreinte
        FROM app.audit
        WHERE cercle_id = $1
        ORDER BY sequence
        "#,
        cercle.uuid()
    )
    .fetch_all(tx.connexion())
    .await?;
    lignes
        .into_iter()
        .map(|l| {
            Ok(EntreeAudit {
                cercle,
                sequence: l.sequence,
                acteur: Acteur::depuis_codes(&l.acteur_type, l.acteur_id)?,
                action: Action::depuis_code(&l.action)?,
                objet: Objet::depuis_codes(&l.objet_type, l.objet_id)?,
                survenu_le: l.survenu_le.to_jiff(),
                empreinte_precedente: Empreinte::depuis_octets(&l.empreinte_precedente)?,
                empreinte: Empreinte::depuis_octets(&l.empreinte)?,
            })
        })
        .collect()
}
