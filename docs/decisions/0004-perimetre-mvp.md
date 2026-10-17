# 0004. Périmètre du MVP

- Statut : Acceptée (le choix B2C reste à confirmer par le porteur)
- Date : 2026-10-17

## Contexte

Le catalogue de fonctionnalités est vaste. Il faut un MVP qui prouve la
valeur vite, sans logistique lourde ni parcours juridiques complexes.

## Décision

- **Clients** : familles en direct (B2C, l'aidant paie) pour le pilote.
- **Équipement** : le **téléphone habituel** du senior et un **espace
  famille web** responsive (Axum, Askama, htmx), installable sur l'écran
  d'accueil du téléphone. **Pas de tablette** ni d'application mobile native
  au MVP.
- **Public** : niveaux d'accompagnement 2 et 3, seniors **capables de
  consentir seuls**.
- **Pas de téléassistance** : la cascade d'alerte s'arrête aux proches
  autorisés ; PPS n'est pas un service d'urgence et le dit clairement.

## Conséquences

- Ni gestion de flotte, ni code mobile, ni fonctionnalités B2B dans le code
  du MVP. Le cercle de soin reste la frontière d'isolation des données ; une
  notion d'« organisation » pourra être ajoutée au-dessus plus tard.
- Notifications par e-mail, SMS et Web Push, sans donnée de santé.
- Les parcours avec représentant légal (tutelle, mandat) viendront après.
