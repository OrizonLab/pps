# 0008. Chiffrement applicatif des données sensibles

- Statut : Acceptée
- Date : 2026-10-17

## Décision

- **Chiffré** : le texte libre et le contenu de santé dont aucune requête n'a
  besoin (médicaments et posologies, notes, transcriptions, résumés,
  messages, fichiers du stockage objet).
- **En clair** : états, horaires, statut des prises, identifiants,
  coordonnées nécessaires au fonctionnement.
- **Chiffrement d'enveloppe** : une clé de données par cercle, elle-même
  chiffrée par une clé maîtresse.
- **AES-256-GCM** (crate `aes-gcm`, RustCrypto), nonce aléatoire de 96 bits ;
  données associées authentifiées = table, colonne, ligne, cercle et version
  de clé (un chiffré ne peut pas être déplacé).
- Format versionné ; crates `secrecy` et `zeroize` ; un type `Sensible<T>`
  dont l'affichage de débogage masque la valeur.

## Conséquences

- Clé maîtresse au MVP : 32 octets en secret Docker, une par environnement,
  avec une copie hors ligne rangée à part des sauvegardes. La perdre, c'est
  perdre les données.
- Ensuite : OpenBao (moteur Transit) derrière un trait `FournisseurCles`.
- Rotation : rechiffrer les clés de données pour la clé maîtresse ; nouvelle
  version et rechiffrement progressif pour une clé de données.
- L'effacement par destruction de clé n'est complet qu'à l'expiration des
  sauvegardes : à écrire dans l'analyse d'impact.
