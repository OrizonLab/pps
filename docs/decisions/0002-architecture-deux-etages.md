# 0002. Architecture en deux étages

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

Un rappel de médicament ou une alerte ne doit jamais dépendre de la
disponibilité ou du comportement d'un modèle d'IA. Les agents IA peuvent se
tromper et ne doivent jamais agir seuls sur des données de santé.

## Décision

- **Étage 1, le moteur de soin** : rappels, check-in, cascades d'alerte,
  consentements, droits d'accès. Code déterministe et testé, sans IA. Il
  détient la vérité.
- **Étage 2, les agents IA** : ils n'accèdent qu'à des outils typés et
  limités exposés par l'étage 1. Ils proposent ; l'activation de toute
  action sensible exige une validation humaine, garantie par le système de
  types quand c'est possible.
- Hermes Agent n'est pas retenu comme orchestrateur (agent mono-utilisateur,
  auto-modifiant, avec accès terminal) : voir
  `docs/08-faisabilite-et-choix-ia.md`.

## Conséquences

- Si l'IA est indisponible, les rappels partent quand même (SMS, message
  enregistré).
- Aucun agent IA n'a d'accès direct à la base de données.
