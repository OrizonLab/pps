# 0001. Rust, Docker et PostgreSQL

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

PPS traite des données de santé de personnes vulnérables. Le porteur du
projet a fixé trois contraintes : Rust pour la sécurité mémoire, Docker pour
le déploiement, PostgreSQL sauf meilleure alternative.

## Décision

- **Rust** pour tout notre code (serveur et cœur partagé des applications).
- **Docker** pour tous les services, avec des images minimales, non root.
- **PostgreSQL** comme unique base de données serveur. L'évaluation des
  alternatives (MongoDB, SurrealDB, CockroachDB, bases spécialisées) est dans
  `docs/06-architecture-technique.md` : aucune ne fait mieux pour nos besoins.

## Conséquences

- Les composants d'infrastructure éprouvés écrits dans d'autres langages
  (PostgreSQL, LiveKit, Temporal) sont utilisés tels quels, dans des
  conteneurs isolés.
- La voix temps réel demande de construire nous-mêmes une partie de la
  logique conversationnelle en Rust.
- Les interfaces mobiles utilisent Flutter, avec la logique sensible en Rust.
