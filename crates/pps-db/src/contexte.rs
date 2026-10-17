//! Transactions limitées à un cercle (décision 0007).
//!
//! Toute lecture ou écriture des données d'un cercle passe par un
//! [`TxCercle`] : la transaction fixe le contexte (`app.cercle_id`, et
//! `app.utilisateur_id` le cas échéant) que les politiques de Row-Level
//! Security utilisent. Les fonctions d'accès aux données prennent un
//! `&mut TxCercle`, jamais le pool.

use pps_domaine::ids::{IdCercle, IdUtilisateur};
use sqlx::{PgConnection, PgPool, Postgres, Transaction};

use crate::erreur::{ErreurAcces, ErreurDonnees};
use crate::membres::{self, MembreLu};

/// Transaction dont toutes les requêtes sont limitées à un cercle.
///
/// Elle est annulée si elle n'est pas validée avec [`TxCercle::valider`].
#[derive(Debug)]
pub struct TxCercle {
    tx: Transaction<'static, Postgres>,
    cercle: IdCercle,
}

impl TxCercle {
    /// Ouvre une transaction pour un utilisateur, qui doit être membre actif
    /// du cercle. Renvoie aussi son appartenance.
    pub async fn pour_membre(
        bdd: &PgPool,
        utilisateur: IdUtilisateur,
        cercle: IdCercle,
    ) -> Result<(Self, MembreLu), ErreurAcces> {
        let mut tx = Self::ouvrir(bdd, cercle, Some(utilisateur)).await?;
        let membre = membres::actif_par_utilisateur(&mut tx, utilisateur)
            .await?
            .ok_or(ErreurAcces::NonMembre)?;
        Ok((tx, membre))
    }

    /// Ouvre une transaction pour créer un nouveau cercle, au nom de son
    /// organisateur.
    pub async fn pour_nouveau_cercle(
        bdd: &PgPool,
        organisateur: IdUtilisateur,
        cercle: IdCercle,
    ) -> Result<Self, ErreurDonnees> {
        Ok(Self::ouvrir(bdd, cercle, Some(organisateur)).await?)
    }

    /// Ouvre une transaction **sans vérifier d'appartenance**, pour le
    /// moteur de soin uniquement : actions du senior au téléphone, ou
    /// traitements automatiques. Ne jamais l'utiliser pour une requête
    /// venant de l'espace famille.
    pub async fn pour_moteur_de_soin(
        bdd: &PgPool,
        cercle: IdCercle,
    ) -> Result<Self, ErreurDonnees> {
        Ok(Self::ouvrir(bdd, cercle, None).await?)
    }

    /// Le cercle auquel la transaction est limitée.
    pub fn cercle(&self) -> IdCercle {
        self.cercle
    }

    /// Valide la transaction.
    pub async fn valider(self) -> Result<(), ErreurDonnees> {
        Ok(self.tx.commit().await?)
    }

    pub(crate) fn connexion(&mut self) -> &mut PgConnection {
        &mut self.tx
    }

    async fn ouvrir(
        bdd: &PgPool,
        cercle: IdCercle,
        utilisateur: Option<IdUtilisateur>,
    ) -> Result<Self, sqlx::Error> {
        let mut tx = bdd.begin().await?;
        fixer_cercle(&mut tx, cercle).await?;
        if let Some(utilisateur) = utilisateur {
            fixer_utilisateur(&mut tx, utilisateur).await?;
        }
        Ok(Self { tx, cercle })
    }
}

// Le troisième argument de `set_config` (`true`) limite la valeur à la
// transaction : elle ne fuit jamais vers la requête suivante qui réutilisera
// la connexion du pool.

pub(crate) async fn fixer_cercle(
    connexion: &mut PgConnection,
    cercle: IdCercle,
) -> Result<(), sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT set_config('app.cercle_id', $1, true)",
        cercle.to_string()
    )
    .fetch_one(connexion)
    .await?;
    Ok(())
}

pub(crate) async fn fixer_utilisateur(
    connexion: &mut PgConnection,
    utilisateur: IdUtilisateur,
) -> Result<(), sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT set_config('app.utilisateur_id', $1, true)",
        utilisateur.to_string()
    )
    .fetch_one(connexion)
    .await?;
    Ok(())
}
