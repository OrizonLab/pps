# 0003. Hébergement

- Statut : Proposée (à confirmer par le porteur du projet)
- Date : 2026-10-17

## Contexte

Le porteur envisage Hostinger. La loi impose un hébergeur **certifié HDS**
pour stocker des données de santé pour le compte de tiers (Code de la santé
publique, article L.1111-8), dans l'Union européenne depuis le
26 septembre 2026. Aucune certification HDS n'a été trouvée pour Hostinger.

## Décision proposée

- **Hostinger** pour ce qui ne contient **aucune donnée de santé réelle** :
  environnement de démonstration, préproduction avec données fictives, site
  vitrine.
- **Production** (dès le premier pilote avec de vraies familles) chez un
  hébergeur **certifié HDS** en France, à choisir avant la fin du MVP.
- Tout est conteneurisé (Docker) : changer d'hébergeur n'a pas d'impact sur
  le code.

## Conséquences

- Aucune donnée réelle de senior ne doit être saisie sur un environnement
  Hostinger.
- La décision devient bloquante avant le premier pilote avec de vraies
  familles. Si Hostinger obtient ou démontre une certification HDS, cette
  décision sera revue.
