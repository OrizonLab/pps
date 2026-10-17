# 0009. Accès à PostgreSQL avec SQLx

- Statut : Acceptée
- Date : 2026-10-17

## Décision

SQLx 0.9, **macros vérifiées à la compilation**, cache `.sqlx/` versionné.

## Conséquences

- Les erreurs de schéma apparaissent à la compilation.
- L'image Docker se construit sans base (`SQLX_OFFLINE=true`).
- La CI applique les migrations sur PostgreSQL 18 et vérifie que le cache
  est à jour (`cargo sqlx prepare --check`).
- SQL compatible PostgreSQL 16 tant que les tests locaux y tournent (UUID v7
  générés en Rust, pas avec `uuidv7()`).
- Les requêtes non vérifiées (`sqlx::query()` et assimilées) sont interdites
  par la règle Clippy `disallowed-methods`, sauf exception justifiée.
