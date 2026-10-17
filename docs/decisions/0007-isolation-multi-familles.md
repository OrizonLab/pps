# 0007. Isolation des familles dans PostgreSQL

- Statut : Acceptée
- Date : 2026-10-17

## Décision

- Le schéma applicatif appartient au rôle `pps_proprietaire` (sans
  connexion) ; les migrations sont exécutées par un rôle membre de celui-ci,
  via une commande `migrer` dédiée.
- Les services se connectent avec des rôles **sans superutilisateur, sans
  `BYPASSRLS`**, qui ne possèdent pas les tables.
- Chaque table portant un `cercle_id` a la Row-Level Security **activée et
  forcée**, avec une politique
  `cercle_id = NULLIF(current_setting('app.cercle_id', true), '')::uuid`.
- Côté Rust, une transaction limitée à un cercle (`TxCercle`) ne s'obtient
  qu'après vérification de l'appartenance de l'utilisateur au cercle ; elle
  fixe la variable avec `set_config('app.cercle_id', ..., true)`. Les
  fonctions d'accès aux données prennent cette transaction, jamais le pool.
- Le journal d'audit n'accepte qu'insertion et lecture, et chaque entrée est
  chaînée à la précédente par empreinte.

## Conséquences et pièges

- Jamais de `SET` au niveau de la session (il fuirait vers la requête
  suivante du pool) ; `set_config(..., true)` dans une transaction explicite.
- Sans contexte, une requête ne renvoie aucune ligne (d'où le `NULLIF`).
- Les vues sont créées avec `security_invoker`.
- Les tests ne s'exécutent jamais en superutilisateur pour la RLS : ils
  utilisent le rôle applicatif et vérifient l'étanchéité entre deux cercles.
- Un test vérifie que toute table du schéma a la RLS activée, forcée, et au
  moins une politique.
- Un service refuse de démarrer si son rôle est superutilisateur ou a
  `BYPASSRLS`.
- Les consentements par catégorie et par proche sont vérifiés dans le
  domaine Rust, côté serveur, à chaque lecture.
