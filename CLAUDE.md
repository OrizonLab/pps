# PPS — Parlons & Prenons Soin : règles du projet

Ce fichier fixe les règles d'ingénierie du dépôt. Il s'applique à toute
personne (ou tout agent) qui contribue au code. La vision et le cadrage
produit sont dans `docs/`, les décisions dans `docs/decisions/` (elles font
référence en cas de doute).

## Ce que nous construisons

Une application par abonnement qui aide les personnes âgées à rester
autonomes chez elles tout en gardant le lien avec leur famille. Nos
utilisateurs sont vulnérables et nous traitons des **données de santé** :
la sécurité, la fiabilité et le respect de la personne passent avant la
vitesse de développement.

## Les principes non négociables

1. **Deux étages** (décision 0002) : le moteur de soin (rappels, check-in,
   alertes, consentements, droits) ne dépend jamais de l'IA. Les agents IA
   **lisent** des données déjà filtrées, **proposent** ou **signalent** ; ils
   n'exécutent rien eux-mêmes.
2. **Le senior décide** : tout partage de donnée passe par un consentement
   explicite, granulaire et révocable, **vérifié côté serveur à chaque
   lecture** (voir `docs/10-suivi-sante-a-distance.md`).
3. **Aucune donnée réelle hors de la production HDS** : développement,
   préproduction, démonstrations et tests utilisent uniquement des données
   fictives (décision 0003).
4. **Pas de donnée de santé** dans les journaux, les traces, les messages
   d'erreur renvoyés au client, les notifications (SMS, e-mail, push), les
   noms de fichiers ou le contenu des tâches planifiées.
5. **Tout accès à une donnée de santé est journalisé** dans le journal
   d'audit, consultable par le senior.
6. **Rien d'irréversible sans validation humaine** (envoi, paiement,
   activation d'un planning de traitement).
7. **Les contenus externes sont non fiables** : ordonnances, courriers, SMS
   et messages peuvent contenir des instructions malveillantes (injection de
   prompt). Ils ne déclenchent jamais d'action par eux-mêmes.
8. **Fiable ne veut pas dire infaillible** : on ne promet jamais qu'une
   alerte ne sera pas manquée, et PPS n'est pas un service d'urgence. Le
   moteur de soin prévoit un mode dégradé sans IA (message enregistré,
   réponse par les touches du téléphone).

## Stack

- **Rust** (édition 2024, toolchain épinglée dans `rust-toolchain.toml`) pour
  tout le code serveur et le cœur des applications. Les interfaces mobiles
  (V2) seront en Flutter, **sans logique métier** : toute règle métier reste
  en Rust.
- **PostgreSQL 18** en Docker, en CI et en production. Le SQL doit rester
  compatible avec PostgreSQL 16, utilisé pour les tests locaux de l'agent de
  développement.
- **Docker** : images épinglées par empreinte, distroless, non root,
  analysées par Trivy (décision 0014).

## Langue (décision 0010)

- Identifiants, tables (au singulier), colonnes et routes : **français en
  ASCII**, sans accents (`PlanningValide`, `consentement.cree_le`).
- Commentaires, documentation, messages d'erreur : français accentué.
- Commits : **Conventional Commits**, type en anglais, description en
  français (`feat(soin): cascade d'alerte`).

## Règles de code Rust

- `unsafe` est **interdit** (`unsafe_code = "forbid"` au niveau du
  workspace). Seule exception prévue : le futur crate de liaison avec les
  applications mobiles, isolé, en `deny`, avec des commentaires `// SAFETY:`
  et une revue obligatoire.
- Clippy strict : `cargo clippy --workspace --all-targets -- -D warnings`
  doit passer. Lints imposés en plus : `unwrap_used`, `expect_used`, `panic`,
  `todo`, `unimplemented`, `dbg_macro`, `indexing_slicing` (autorisés dans
  les tests uniquement). Pas de `#[allow(...)]` sans commentaire qui
  justifie.
- **Erreurs typées** avec `thiserror`. Une erreur interne n'expose jamais de
  détail technique ou de donnée personnelle au client HTTP.
- **Types métier forts** : identifiants en newtypes, états en types
  distincts (typestate) quand une règle de sécurité en dépend, validation à
  la frontière (une valeur invalide ne doit pas pouvoir être construite).
- Données sensibles dans des types qui masquent leur valeur à l'affichage de
  débogage.
- Dates : instants en UTC (`timestamptz`), heure locale calculée à partir du
  fuseau du senior, avec jiff (décision 0011).
- Identifiants : UUID v7 générés côté Rust.
- Toute nouvelle dépendance doit être justifiée (utile, maintenue, licence
  compatible) et passer `cargo deny check`.

## Base de données (décisions 0007 et 0009)

- Requêtes avec les **macros SQLx vérifiées à la compilation** ; le cache
  `.sqlx/` est versionné et tenu à jour (`cargo sqlx prepare --workspace`).
  Les requêtes non vérifiées sont interdites sauf exception justifiée. Pas de
  SQL construit par concaténation.
- Migrations dans `migrations/`, **jamais modifiées** une fois fusionnées sur
  `main`. Elles sont exécutées par la commande `migrer`, jamais par un
  service au démarrage.
- Les services se connectent avec un rôle **sans** droits de propriétaire,
  **sans** superutilisateur ni `BYPASSRLS`.
- Toute table liée à un cercle a la **Row-Level Security activée et
  forcée**. Le cercle est fixé par transaction (`set_config(..., true)`),
  jamais au niveau de la session. Le code accède aux données d'un cercle
  uniquement via une transaction limitée à ce cercle.

## Tests

- Tests unitaires au plus près du domaine, sans base.
- Tests d'intégration contre un **vrai PostgreSQL** (jamais de simulacre de
  base pour tester le SQL), avec le rôle applicatif (pas en
  superutilisateur) pour tout ce qui touche à la RLS.
- Chaque bug corrigé s'accompagne d'un test qui l'aurait détecté.
- On ne désactive, n'ignore ni ne supprime jamais un test pour rendre la CI
  verte.
- Données de test **fictives** uniquement.

## Commandes

```sh
cargo fmt --all                                         # formatage
cargo clippy --workspace --all-targets -- -D warnings   # lints
cargo test --workspace                                  # tests (DATABASE_URL requis pour l'intégration)
cargo deny check                                        # dépendances : failles, licences, sources
cargo sqlx prepare --workspace                          # mise à jour du cache des requêtes
docker compose up --build                               # environnement complet
```

## Façon de travailler

- **Un incrément = une PR** depuis la branche de travail vers `main`, dans
  l'ordre décrit dans `docs/07-feuille-de-route.md`.
- Avant d'ouvrir la PR : formatage, Clippy, tests et `cargo deny` passent en
  local.
- Chaque PR est **relue par un sous-agent réviseur** (sécurité, exactitude,
  qualité, conformité aux règles ci-dessus). Ses remarques bloquantes sont
  corrigées avant fusion, puis la revue est relancée.
- Fusion sur `main` en **squash** (décision 0012), quand la CI est verte et
  la revue validée.
- Toute décision technique ou fonctionnelle notable est consignée dans
  `docs/decisions/`.
- On avance par petites tranches verticales et utiles. Pas de code
  spéculatif « pour plus tard ».
