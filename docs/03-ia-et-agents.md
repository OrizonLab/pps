# 3. L'IA et les agents : une équipe invisible au service de la famille

Le service étant payant, on peut investir dans une IA de très haut niveau.
L'idée n'est pas d'avoir « un chatbot », mais une **équipe d'agents
spécialisés**, coordonnés, qui travaillent en coulisses pour le senior et sa
famille. Pour le senior, il n'y a qu'**un seul interlocuteur** : son
compagnon, avec un prénom et une voix qu'il a choisis.

## L'architecture des agents

```mermaid
flowchart TD
    S["Senior<br/>(téléphone, tablette, TV, voix)"] <--> C["Compagnon<br/>(interlocuteur unique)"]
    C --> O{"Orchestrateur"}
    O --> SA["Agent Santé"]
    O --> CO["Agent Coordinateur famille"]
    O --> AD["Agent Administratif"]
    O --> SE["Agent Sentinelle"]
    O --> TU["Agent Tuteur numérique"]
    O --> BI["Agent Biographe"]
    M[("Mémoire personnelle<br/>chiffrée")] --- O

    O -- "outils typés :<br/>lire, proposer, signaler" --> MS["Moteur de soin (étage 1, sans IA)<br/>rappels, check-in, alertes,<br/>consentements, droits"]
    H["Validation humaine<br/>pour toute action sensible"] --> MS
    MS --> F["Famille et aidants<br/>(espace famille)"]
    MS --> P["Professionnels<br/>(cahier de liaison)"]
```

**Répartition des rôles** (voir la
[décision 0002](decisions/0002-architecture-deux-etages.md)) : les rappels, le
check-in, les cascades d'alerte et le filtrage selon les consentements sont
**exécutés par le moteur de soin**, sans IA. Les agents ne font que **lire**
les données que le moteur leur transmet (déjà filtrées selon les
consentements), **proposer** (un planning, un résumé) ou **signaler** un
événement (« le senior a dit être tombé »). C'est le moteur qui décide quoi
faire et qui prévenir.

| Agent | Rôle | Exemples | Limites strictes |
|---|---|---|---|
| **Compagnon** | L'interlocuteur unique du senior. Conversation naturelle, mémoire des prénoms, des goûts, de l'histoire de vie. | Appel du matin, lecture des messages de la famille, discussion sur le match d'hier. | Dit toujours qu'il est une IA. Ne se fait jamais passer pour un proche. |
| **Santé** | Aide à la conversation sur la santé : relaie les rappels décidés par le moteur de soin, tient le journal de ce que le senior dit (douleur, sommeil), prépare la fiche de consultation. | « C'est l'heure de votre comprimé du midi », fiche de préparation de consultation. | **Jamais de diagnostic, jamais de modification de traitement.** Il ne programme ni n'annule aucun rappel : il peut seulement proposer. Toute ordonnance lue est validée par un humain. |
| **Coordinateur famille** | Rédaction des synthèses, suggestions de répartition des tâches. | Résumé du dimanche, « Qui accompagne mamie jeudi ? ». | Ne reçoit que des données déjà filtrées par le moteur de soin selon les consentements. Ne diffuse rien lui-même. |
| **Administratif** | Lecture des courriers, aide aux démarches, échéances. | « Ce courrier de la CAF demande une réponse avant le 12. » | Prépare, n'envoie jamais rien sans validation. Aucun paiement autonome. |
| **Sentinelle** | Sécurité : analyse des messages suspects (anti-arnaque), signalement d'événements au moteur de soin. | Analyse d'un SMS suspect ; signale « le senior a dit être tombé ». | N'exécute ni check-in ni cascade : c'est le moteur de soin qui le fait, selon les seuils et destinataires choisis par le senior et sa famille. |
| **Tuteur numérique** | Apprentissage, guidage pas à pas, lecture d'écran. | « Appuyez sur le rond vert en bas à droite. » | Ne saisit jamais un mot de passe ou un code bancaire à la place du senior. |
| **Biographe** | Recueil de souvenirs, livre de vie, arbre généalogique. | Questions au fil des conversations, montage du livre annuel. | Le senior relit et choisit ce qui est partagé. |

## Les règles d'or de l'IA chez PPS

1. **Transparence** : l'IA dit qu'elle est une IA, à chaque fois que c'est
   utile, et en particulier au début de chaque appel. C'est aussi une
   obligation du règlement européen sur l'IA (AI Act).
2. **L'humain dans la boucle** : aucune action irréversible ou sensible
   (envoi d'un document, paiement, partage d'une donnée de santé, changement
   de planning de traitement) sans validation humaine.
3. **Pousser vers l'humain** : la réussite de l'IA se mesure au nombre de
   contacts humains qu'elle provoque. Elle propose d'appeler un proche,
   relaie les nouvelles, rappelle les anniversaires. Elle évite toute
   dépendance émotionnelle.
4. **Pas de clonage de voix de proches** pour parler au senior. C'est une
   ligne rouge : confusion, manipulation, et c'est exactement la technique
   des arnaqueurs.
5. **Pas de manipulation, pas de publicité** : l'IA ne vend rien, ne pousse à
   aucun achat, ne cherche pas à maximiser le temps d'utilisation.
6. **Respect du refus** : « Je n'ai pas envie de parler aujourd'hui » est une
   réponse valable. L'IA s'adapte, sans culpabiliser.
7. **Orientation en cas de propos alarmants** : si le senior **dit**
   explicitement avoir mal à la poitrine, être tombé, ou tient des propos
   suicidaires, l'IA lui conseille d'appeler le 15 ou le 112 (ou le 3114),
   propose de prévenir un proche, et signale l'événement au moteur de soin.
   Elle n'évalue jamais l'état clinique de la personne (ce serait une
   fonction de dispositif médical). Le protocole est rédigé avec des
   professionnels de santé.
8. **Explicabilité pour la famille** : chaque alerte dit pourquoi elle a été
   déclenchée (« Pas de réponse à l'appel de 9h, ni à celui de 9h30 »).
9. **Pas d'analyse des émotions à partir de la voix** : l'IA retient ce que
   le senior **dit** (« je suis fatiguée »), jamais une émotion déduite du ton
   de sa voix. C'est plus respectueux, et la reconnaissance des émotions est
   classée « à haut risque » par le règlement européen sur l'IA.
10. **Ni terminal, ni auto-modification** : les agents n'appellent que des
    outils typés et limités, et leurs instructions sont versionnées et
    testées comme du code (voir le [document 8](08-faisabilite-et-choix-ia.md)).

## Les briques technologiques

| Brique | Usage | Points d'attention |
|---|---|---|
| **Grand modèle de langage (LLM)** | Conversation, compréhension, synthèse, rédaction | Architecture multi-fournisseurs (par exemple Anthropic Claude, Mistral). Aucune donnée de santé n'est envoyée à un service qui n'est pas couvert par un certificat HDS et dont le traitement ne reste pas dans l'EEE. Pas d'entraînement sur les données des utilisateurs. |
| **Reconnaissance vocale (STT)** | Comprendre le senior | À tester sur des voix âgées : débit lent, voix tremblante, accents régionaux, appareils dentaires. Un vrai sujet de qualité. |
| **Synthèse vocale (TTS)** | Faire parler le compagnon | Voix chaleureuse, naturelle, débit réglable, pas de voix « robot ». |
| **Voix temps réel** | Appels téléphoniques fluides | Latence faible, gestion des interruptions et des silences longs. |
| **Téléphonie** | Appels sortants et entrants sur fixe ou mobile | Opérateur ou plateforme de téléphonie programmable, numéros français. |
| **Vision** | Lire une ordonnance, un courrier, un écran, une boîte de médicament | Toujours avec confirmation humaine pour la santé. |
| **Mémoire personnelle** | Se souvenir de la personne, de sa famille, de son histoire | Stockage chiffré, consultable et effaçable par le senior. |
| **Garde-fous** | Filtrage des sujets à risque, détection d'urgence | Tests réguliers et scénarios rédigés avec des gériatres. |

**Règle commune à tous les sous-traitants** (modèles d'IA, reconnaissance et
synthèse vocales, téléphonie, SMS) : un service ne reçoit des données de santé
que si son offre précise est couverte par un certificat HDS et que le
traitement reste dans l'Espace économique européen. Sinon, il ne reçoit que
des données sans caractère de santé, ou il n'est pas utilisé. Les contenus
externes (ordonnances, courriers, SMS, messages) sont traités par les agents
comme **non fiables** : ils peuvent contenir des instructions malveillantes
(injection de prompt) et ne doivent jamais déclencher d'action par eux-mêmes.

## Évaluer l'IA sérieusement

Parce que les utilisateurs sont vulnérables, la qualité de l'IA doit être
mesurée en continu :

- **Jeux de tests** de conversations réelles (anonymisées et consenties) :
  urgences, confusions, arnaques, questions médicales.
- **Évaluation par des professionnels** (gériatres, infirmières, psychologues)
  sur un échantillon de conversations.
- **Indicateurs suivis** : taux de compréhension de la parole, fausses
  alertes, alertes manquées, satisfaction du senior, nombre de contacts
  humains générés.
- **Panel de seniors testeurs** rémunérés, impliqués dès le début.
