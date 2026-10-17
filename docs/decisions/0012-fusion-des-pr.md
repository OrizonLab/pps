# 0012. Fusion des PR

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

Chaque incrément est relu puis fusionné sur `main` ; l'historique doit
rester lisible et chaque état de `main` doit passer la CI.

## Décision

**Squash merge** sur `main`. Titre de la PR au format Conventional Commits ;
corps : contexte, décisions et verdict de la revue.

## Conséquences

- Un commit par incrément, qui passe la CI ; historique linéaire, `revert` et
  `bisect` simples.
- Recommandation : protéger `main` (CI verte obligatoire, historique
  linéaire, pas de force-push).
