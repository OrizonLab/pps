# 0007. Isolation des familles dans PostgreSQL

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

Des milliers de familles partagent la même base. Une seule requête mal
écrite ne doit jamais exposer les données d'une famille à une autre.

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
  chaînée à la précédente du même cercle par empreinte.

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

## Tables transverses (à détailler dans l'incrément 2)

Certaines lectures ont lieu **avant** de connaître le cercle, ou portent sur
**plusieurs** cercles. Elles ne peuvent pas suivre la politique standard ;
chacune aura une politique dédiée, documentée et testée :

- **sessions** : retrouver la session à partir du jeton haché, avant de
  connaître le cercle (politique fondée sur l'utilisateur, pas sur le cercle) ;
- **appartenance aux cercles** : vérifier qu'un utilisateur est membre d'un
  cercle avant d'ouvrir la transaction de ce cercle ;
- **tâches planifiées** : le planificateur réserve des tâches de tous les
  cercles, avec un **rôle dédié** (`pps_soin`) limité à cette table, puis
  exécute chaque tâche dans la transaction de son cercle.

Le test « toute table a la RLS activée et forcée » reste valable : ces tables
ont aussi la RLS, avec leurs propres politiques. Le journal d'audit est
chaîné **cercle par cercle**.
