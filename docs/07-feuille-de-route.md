# 7. Feuille de route, indicateurs et questions ouvertes

## La recommandation : commencer par « l'appel du matin »

Le catalogue est vaste. Pour réussir, il faut une **porte d'entrée** simple,
qui apporte une valeur immédiate à la fois au senior (qui l'utilise) et à
l'aidant (qui paie souvent).

**Proposition : l'appel quotidien par l'IA, sur n'importe quel téléphone.**

Chaque matin, à l'heure choisie, le compagnon appelle le senior :

1. Il prend des nouvelles et vérifie que tout va bien (le « Bonjour »).
2. Il rappelle les médicaments et les rendez-vous du jour.
3. Il lit les messages et décrit les photos envoyées par la famille.
4. Il recueille ce que le senior veut transmettre (« Dis à Sophie que… »).
5. Il envoie un court résumé à la famille, et une alerte si quelque chose ne
   va pas.

Pourquoi c'est la bonne porte d'entrée :

- **Aucun équipement, aucun apprentissage** : ça marche avec un fixe.
- **Valeur immédiate pour les deux** : le senior a quelqu'un à qui parler,
  l'aidant est rassuré chaque jour.
- **Démontre la puissance de l'IA** sans jamais la rendre intimidante.
- **Base naturelle pour tout le reste** : gazette, appli famille, tablette,
  académie numérique viennent ensuite enrichir le même service.

**Attention à la concurrence** : un service d'appels IA aux seniors existe
déjà en France (voir le [marché](04-marche-et-modele-economique.md)). L'appel
seul ne suffit donc pas. Dès le MVP, il faut livrer la **boucle complète** :
l'appel du matin, l'appli famille où chacun poste et répond, le résumé du
dimanche et les rappels santé avec escalade. C'est cette boucle « senior +
famille + soin » qui nous distingue.

## Les phases

### Phase 0 — Découverte (mois 0 à 3)

- **Entretiens** : 30 familles aidantes, 20 seniors (de niveaux d'autonomie
  différents), 10 professionnels (aides à domicile, infirmières, médecins,
  CCAS).
- **Test « magicien d'Oz »** : l'appel du matin réalisé par un humain qui suit
  un script, pour valider l'usage et le prix avant de coder. Avec le
  consentement éclairé des participants, un minimum de données, et aucune
  donnée de santé saisie dans un outil non conforme (tableur partagé,
  messagerie grand public).
- **Banc d'essai de la reconnaissance vocale** : collecte de voix âgées
  enregistrées avec consentement, en qualité téléphonique, pour comparer les
  moteurs de reconnaissance avant de construire l'agent vocal (voir la
  [décision 0013](decisions/0013-voix-temps-reel.md)).
- **Prototype cliquable** de l'appli famille et du résumé quotidien.
- **Cadrage juridique** : données de santé, hébergement certifié, statut
  « services à la personne », règlement européen sur l'IA.
- **Choix du nom et de l'identité** (PPS comme nom de code).

### Phase 1 — MVP (mois 3 à 6)

Le périmètre est fixé par la [décision 0004](decisions/0004-perimetre-mvp.md) :
familles en direct (B2C), téléphone habituel du senior, espace famille web,
niveaux d'accompagnement 2 et 3, seniors capables de consentir seuls.

- Appel du matin sur le fixe ou le mobile : d'abord **scripté** (voix de
  synthèse, réponses par oui/non ou par les touches), puis **conversationnel**
  avec l'IA.
- Check-in quotidien et cascade d'alerte **limitée aux proches autorisés** :
  pendant le pilote, PPS n'est ni un service de téléassistance ni un service
  d'urgence, et le dit clairement.
- Rappels de médicaments et de rendez-vous, avec un planning **saisi par un
  proche et validé** par un humain habilité.
- Espace famille web (installable sur le téléphone) : messages et photos lus
  au senior pendant l'appel, agenda partagé, tableau de bord « Comment
  va-t-il ? », résumé hebdomadaire.
- **Pilote avec 50 familles**, suivi de près, **uniquement chez un hébergeur
  certifié HDS**.

### Phase 2 — Élargissement (mois 6 à 12)

- Tablette simplifiée préconfigurée (en location), visio.
- Lecture de l'ordonnance par photo, résumé rédigé par l'IA, gazette papier.
- Compagnon numérique (questions sur le téléphone, la tablette).
- Bouclier anti-arnaque et copilote administratif (courriers).
- Biographe IA et livre de vie.
- Académie numérique : parcours, bac à sable, prise en main à distance.
- Cahier de liaison avec les professionnels.
- Partenariat téléassistance 24h/24.
- Premier pilote B2B (mutuelle, caisse de retraite ou département).

### Phase 3 — Écosystème (mois 12 à 24)

- Capteurs non intrusifs et jumeau de routine.
- Détection de signaux faibles avec un partenaire clinique (démarche de
  certification dispositif médical si nécessaire).
- Interopérabilité avec Mon Espace Santé.
- Application TV, objet dédié.
- Annuaire et services locaux, vie sociale.
- Ouverture à d'autres pays européens.

## Les premiers incréments de code

Chaque incrément est une PR relue puis fusionnée (voir `CLAUDE.md`). L'ordre
vise une boucle démontrable « appel du matin + rappels + résumé famille »
sans jamais exposer d'API non authentifiée.

1. **Socle** : workspace Rust, images Docker durcies, CI (tests, lints,
   dépendances, analyse des images), point de santé.
2. **Cercle de soin, consentements, journal d'audit** : modèle métier, rôles
   de base de données, Row-Level Security. Aucune route métier exposée.
3. **Authentification** : lien magique, sessions, protection CSRF, accès à la
   base limité au cercle de l'utilisateur, premières pages (créer un cercle,
   inviter un proche).
4. **Chiffrement applicatif** des données sensibles.
5. **Planning de médicaments** : proposition, validation humaine, planning
   validé, avec saisie manuelle.
6. **Planificateur** et calcul des occurrences (fuseau du senior, changements
   d'heure) : premiers rappels réels.
7. **Appel du matin scripté** : check-in, rappels du jour, recueil de
   l'accord seul à seul, cascade d'alerte vers les proches autorisés.
8. **« Comment va-t-il ? »** : tableau de bord, messages de la famille lus
   pendant l'appel, résumé construit sans IA.

Ensuite : clés d'accès (passkeys), agent vocal conversationnel, résumé par
l'IA, lecture d'ordonnance, gestion des clés avec OpenBao, déploiement chez
l'hébergeur HDS.

## Les indicateurs de réussite

**Indicateur principal (« North Star »)** : le nombre de **contacts humains
significatifs par senior et par semaine** (appels, visios, visites,
messages échangés avec la famille et les proches).

| Domaine | Indicateurs |
|---|---|
| **Lien social** | Contacts humains par semaine, nombre de proches actifs dans le cercle, jours sans aucun contact |
| **Santé** | Taux d'observance des traitements, RDV manqués, alertes traitées |
| **Autonomie** | Maintien à domicile, progression dans l'académie numérique |
| **Aidants** | Charge ressentie (questionnaire type échelle de Zarit), temps gagné, sérénité déclarée |
| **Qualité de l'IA** | Compréhension de la parole, fausses alertes, alertes manquées |
| **Business** | Conversion après essai, rétention à 3, 6 et 12 mois, satisfaction (seniors et familles) |

## Les questions ouvertes à trancher ensemble

Les questions 1 à 3 ont désormais une réponse par défaut dans la
[décision 0004](decisions/0004-perimetre-mvp.md), à confirmer par le
porteur du projet.

1. **Qui est le premier client ?** Les familles en direct (B2C), ou un
   partenaire qui distribue (mutuelle, caisse de retraite, département) ?
2. **Matériel ou pas ?** Commencer sans aucun matériel (téléphone seul), ou
   proposer dès le départ une tablette préconfigurée ?
3. **Quel niveau d'autonomie cibler en premier ?** Les niveaux 2 et 3
   (isolés, fragiles) semblent avoir le besoin le plus fort et le plus facile
   à expliquer.
4. **Quelle place pour l'humain ?** Faut-il des conseillers PPS salariés
   (centre d'appel bienveillant), des bénévoles, des partenaires locaux ?
5. **Statut « services à la personne »** : faut-il structurer l'entreprise
   pour ouvrir droit au crédit d'impôt ? Quelles prestations sont éligibles ?
6. **Nom et marque** : garder PPS / « Parlons & Prenons Soin » ?
7. **Équipe fondatrice** : quelles compétences avons-nous déjà (produit,
   tech, santé, commercial) et lesquelles faut-il recruter ?
8. **Financement** : autofinancement, subventions (Bpifrance, appels à projets
   silver économie, conférences des financeurs), levée de fonds ?

## Les prochaines étapes concrètes

1. Relire ce brainstorming, éliminer, ajouter, prioriser.
2. Trancher les questions ouvertes 1 à 3.
3. Rédiger le guide d'entretien et lancer les premières interviews de familles.
4. Monter le test « magicien d'Oz » de l'appel du matin avec 5 à 10 familles.
5. En parallèle, prototyper techniquement l'appel vocal IA en français, pour
   mesurer la qualité de compréhension sur des voix âgées.
