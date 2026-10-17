# PPS — Parlons & Prenons Soin : règles du projet

Ce fichier fixe les règles d'ingénierie du dépôt. Il s'applique à toute
personne (ou tout agent) qui contribue au code. La vision et le cadrage
produit sont dans `docs/`, les décisions techniques dans `docs/decisions/`.

## Ce que nous construisons

Une application par abonnement qui aide les personnes âgées à rester
autonomes chez elles tout en gardant le lien avec leur famille. Nos
utilisateurs sont vulnérables et nous traitons des **données de santé** :
la sécurité, la fiabilité et le respect de la personne passent avant la
vitesse de développement.

## Les principes non négociables

1. **Deux étages** (voir `docs/08-faisabilite-et-choix-ia.md`) : le moteur
   de soin (rappels, check-in, alertes, consentements) ne dépend jamais de
   l'IA. Les agents IA **proposent**, un humain **valide**.
2. **Le senior décide** : tout partage de donnée passe par un consentement
   explicite, granulaire et révocable (voir `docs/10-suivi-sante-a-distance.md`).
3. **Pas de donnée de santé dans les journaux**, les traces, les messages
   d'erreur renvoyés au client ou les noms de fichiers.
4. **Minimisation** : on ne stocke que ce qui sert.
5. **Rien d'irréversible sans validation humaine** (envoi, paiement,
   activation d'un planning de traitement).

## Stack

- **Rust** (édition 2024, toolchain épinglée dans `rust-toolchain.toml`)
  pour tout notre code.
- **PostgreSQL** (18 en Docker et en CI ; le code doit aussi fonctionner
  sur 16, utilisé pour les tests locaux).
- **Docker** pour le déploiement : images distroless, non root.

## Règles de code Rust

- `unsafe` est **interdit** (`unsafe_code = "forbid"` au niveau du workspace).
- Clippy strict : `cargo clippy --workspace --all-targets -- -D warnings`
  doit passer. Pas de `#[allow(...)]` sans commentaire qui justifie.
- **Pas de `unwrap()`, `expect()` ni `panic!` dans le code de production.**
  Autorisés dans les tests uniquement.
- **Erreurs typées** avec `thiserror`. Une erreur interne n'expose jamais de
  détail technique ou de donnée personnelle au client HTTP.
- **Types métier forts** : identifiants en newtypes, états en types
  distincts (typestate) quand une règle de sécurité en dépend, validation à
  la frontière (une valeur invalide ne doit pas pouvoir être construite).
- **Pas de SQL construit par concaténation.** Requêtes paramétrées
  uniquement.
- Dates : instants stockés en UTC (`timestamptz`) ; l'heure locale du senior
  est calculée à partir de son fuseau horaire enregistré.
- Identifiants : UUID v7 générés côté Rust.
- Toute nouvelle dépendance doit être justifiée (utile, maintenue, licence
  compatible) et passer `cargo deny check`.

## Langue

La documentation, les commentaires et les messages de commit sont en
français. La règle pour les identifiants du code (types, fonctions, tables)
est fixée par une décision dédiée dans `docs/decisions/`, prise avant le
premier code.

## Base de données

- Migrations SQLx dans `migrations/`, **jamais modifiées** une fois
  fusionnées sur `main` : on ajoute une nouvelle migration.
- L'application se connecte avec un rôle **sans** droits de propriétaire et
  **sans** `BYPASSRLS`. L'isolation entre familles repose sur la
  Row-Level Security **en plus** des contrôles dans le code.

## Tests

- Tests unitaires au plus près du domaine (`pps-domaine`), sans base.
- Tests d'intégration contre un **vrai PostgreSQL** (jamais de simulacre de
  base pour tester le SQL).
- Chaque bug corrigé s'accompagne d'un test qui l'aurait détecté.
- On ne désactive, n'ignore ni ne supprime jamais un test pour rendre la CI
  verte.

## Commandes

```sh
cargo fmt --all                                         # formatage
cargo clippy --workspace --all-targets -- -D warnings   # lints
cargo test --workspace                                  # tests (DATABASE_URL requis pour l'intégration)
cargo deny check                                        # dépendances : failles, licences, sources
docker compose up --build                               # environnement complet
```

## Façon de travailler

- **Un incrément = une PR** depuis la branche de travail vers `main`.
- Avant d'ouvrir la PR : formatage, clippy, tests et `cargo deny` passent en
  local.
- Chaque PR est **relue par un sous-agent réviseur** (sécurité, exactitude,
  qualité, conformité aux règles ci-dessus). Ses remarques bloquantes sont
  corrigées avant fusion.
- Fusion sur `main` en **squash**, avec un message de commit clair en
  français, quand la CI est verte et la revue validée.
- Toute décision technique ou fonctionnelle notable est consignée dans
  `docs/decisions/` (format court : contexte, décision, conséquences).
- On avance par petites tranches verticales et utiles. Pas de code
  spéculatif « pour plus tard ».
