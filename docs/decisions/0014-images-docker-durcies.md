# 0014. Images Docker : la plus récente, la moins exposée

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

Le porteur demande de privilégier, pour chaque image (PostgreSQL, images de
base), la version la plus récente qui présente le moins de failles connues.

## Décision

Choix fait par analyse Trivy des variantes officielles disponibles
(2026-10-17, failles hautes et critiques) :

| Image | Résultat | Retenue |
|---|---|---|
| `postgres:18-alpine` (PostgreSQL 18.6) | 1 critique, 24 hautes, **toutes dans `gosu`** | **Oui, sans `gosu`** : 0 haute ou critique |
| `postgres:18-trixie` | 2 critiques, 85 hautes | Non |
| `postgres:18-bookworm` | 4 critiques, 95 hautes | Non |
| `distroless/cc-debian13:nonroot` | 0 haute ou critique | **Oui** (exécution des services Rust) |
| `distroless/cc-debian12:nonroot` | 1 haute | Non |

Les images Chainguard et Docker Hardened Images n'étaient pas accessibles
depuis l'environnement de développement (authentification ou réseau) ; elles
pourront être réévaluées.

- Image PostgreSQL dérivée : `gosu` retiré, exécution directe sous
  l'utilisateur `postgres`, jamais root.
- Compilation Rust sur `rust:1.97.0-trixie` (même glibc que l'image
  d'exécution).
- Toutes les images sont **épinglées par empreinte**.
- La CI construit et analyse chaque image : une faille haute ou critique
  corrigeable **bloque** la fusion. Analyse relancée chaque semaine.
- Dependabot propose les mises à jour des empreintes, des dépendances Rust et
  des actions GitHub.

## Conséquences

- PostgreSQL 19 (sortie prévue le 19 novembre 2026) sera adopté après sa
  première version corrective, si l'analyse le confirme.
- Quand pgvector sera nécessaire, il sera compilé dans l'image dérivée, puis
  l'image sera réanalysée.
