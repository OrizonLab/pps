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
