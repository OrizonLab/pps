# Journal des décisions

Chaque décision technique ou fonctionnelle notable est consignée ici, dans un
fichier court et numéroté. Une décision n'est jamais réécrite : si elle
change, une nouvelle décision la remplace et l'ancienne passe au statut
« Remplacée ».

## Format

```markdown
# NNNN. Titre

- Statut : Proposée | Acceptée | Remplacée par NNNN
- Date : AAAA-MM-JJ

## Contexte
## Décision
## Conséquences
```

## Index

| N° | Décision | Statut |
|---|---|---|
| [0001](0001-rust-docker-postgresql.md) | Rust, Docker et PostgreSQL | Acceptée |
| [0002](0002-architecture-deux-etages.md) | Architecture en deux étages : moteur de soin sans IA, agents IA contrôlés | Acceptée |
| [0003](0003-hebergement.md) | Hébergement : Hostinger hors données de santé, production chez un hébergeur HDS | Proposée |
| [0004](0004-perimetre-mvp.md) | Périmètre du MVP : B2C, téléphone et espace famille web, niveaux 2 et 3 | Acceptée (B2C à confirmer) |
| [0005](0005-planification-durable.md) | Planification durable : file de tâches PostgreSQL | Acceptée |
| [0006](0006-authentification.md) | Authentification : lien magique puis passkeys, sessions opaques | Acceptée |
| [0007](0007-isolation-multi-familles.md) | Isolation des familles : rôles séparés et Row-Level Security forcée | Acceptée |
| [0008](0008-chiffrement-applicatif.md) | Chiffrement applicatif : AES-256-GCM, une clé par cercle | Acceptée |
| [0009](0009-acces-postgresql-sqlx.md) | Accès à PostgreSQL : SQLx, macros vérifiées à la compilation | Acceptée |
| [0010](0010-langue-du-code.md) | Langue du code : français en ASCII, Conventional Commits | Acceptée |
| [0011](0011-dates.md) | Dates et fuseaux horaires : jiff | Acceptée |
| [0012](0012-fusion-des-pr.md) | Fusion des PR : squash merge | Acceptée |
| [0013](0013-voix-temps-reel.md) | Voix temps réel : tout en Rust, appel scripté d'abord | Acceptée |
| [0014](0014-images-docker-durcies.md) | Images Docker : la plus récente, la moins exposée | Acceptée |
