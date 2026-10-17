# 0003. Hébergement

- Statut : Proposée (à confirmer par le porteur du projet)
- Date : 2026-10-17

## Contexte

Le porteur envisage Hostinger. La loi impose un hébergeur **certifié HDS**
pour stocker des données de santé pour le compte de tiers (Code de la santé
publique, article L.1111-8), et, depuis le 26 septembre 2026, un hébergement
exclusivement dans l'Union européenne ou l'Espace économique européen.
Aucune certification HDS n'a été trouvée pour Hostinger.

## Décision proposée

- **Hostinger** pour ce qui ne contient **aucune donnée de santé réelle** :
  développement, préproduction et démonstrations avec des données fictives,
  site vitrine.
- **Production** (dès la première donnée réelle d'un senior, même d'une
  famille amie) : une **offre managée chez un hébergeur certifié HDS en
  France**, dont le certificat couvre l'exécution des conteneurs,
  PostgreSQL et les sauvegardes (par exemple Clever Cloud, Scaleway ou
  OVHcloud, à vérifier offre par offre).
- Si PPS administrait elle-même des machines virtuelles et la base, elle
  exercerait des activités du référentiel HDS (administration, sauvegarde) :
  il faudrait une certification propre ou un infogérant certifié. Ce point
  est à faire valider par un juriste.
- **Sous-traitants** (modèles d'IA, reconnaissance et synthèse vocales,
  téléphonie, SMS) : un service ne reçoit des données de santé que si son
  offre est couverte par un certificat HDS et que le traitement reste dans
  l'EEE.
- **Opérateur téléphonique** : l'appel scripté (incrément 7) transporte des
  réponses au check-in. Le juriste doit dire si un simple transport relève de
  l'HDS ; c'est une dépendance de l'incrément 7.
- Tout est conteneurisé : changer d'hébergeur n'a pas d'impact sur le code.

## Conséquences

- Aucune donnée réelle de senior sur un environnement Hostinger.
- Le choix de l'hébergeur HDS devient bloquant avant le premier pilote. Il
  faut le faire deux à trois mois avant (contrat, analyse d'impact,
  délégué à la protection des données, sous-traitants), au plus tard
  pendant l'incrément 6.
