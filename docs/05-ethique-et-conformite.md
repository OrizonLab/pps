# 5. Éthique, confiance et conformité

Nos utilisateurs sont parmi les plus vulnérables, et nous manipulons des
données de santé, de vie privée et d'intimité familiale. **La confiance est
notre premier produit.** Elle doit être pensée dès le premier jour, pas
ajoutée à la fin.

> Les points réglementaires ci-dessous sont un premier repérage pour cadrer
> le projet. Ils devront être validés par un juriste spécialisé (santé et
> numérique) avant le lancement.

## Le cadre réglementaire à anticiper

| Sujet | Ce que ça implique pour PPS |
|---|---|
| **RGPD : données de santé** | Les données de santé sont des données « sensibles » (article 9). Il faut une base légale solide (consentement explicite), une **analyse d'impact (AIPD)** avant le lancement, un délégué à la protection des données (DPO), et un registre des traitements. |
| **Hébergement de données de santé (HDS)** | Dès qu'on héberge des données de santé pour le compte de nos utilisateurs (traitements, rendez-vous médicaux, observations de santé), l'hébergeur doit être **certifié HDS** (Code de la santé publique, article L.1111-8). Le référentiel en vigueur est la **version 2.0** [1]. Depuis le 26 septembre 2026, un décret impose un hébergement **exclusivement dans l'UE ou l'EEE** et encadre strictement tout accès depuis un pays tiers. On choisit donc un hébergeur certifié en France. |
| **Règlement européen sur l'IA (AI Act, 2024/1689)** | **Interdiction**, depuis février 2025, des systèmes d'IA qui exploitent les vulnérabilités liées à l'âge (article 5) : c'est une ligne rouge directement liée à notre public. **Transparence** : depuis le 2 août 2026, les personnes doivent savoir qu'elles parlent à une IA (article 50) [2]. Les contenus générés (voix, images) doivent être identifiables. **Déduire les émotions de la voix** ferait de PPS un système à haut risque : on s'en tient au contenu de ce que dit le senior. Les obligations « haut risque » s'appliqueront à partir de décembre 2027 (annexe III) et d'août 2028 (IA intégrées à un dispositif médical). Enfin, les équipes doivent être formées à l'IA (article 4). |
| **Dispositif médical (règlement européen 2017/745)** | Un logiciel qui revendique une finalité médicale (**diagnostiquer, prévenir, surveiller, prédire ou traiter** une maladie) devient un dispositif médical, avec marquage CE et certification. La plupart des logiciels médicaux sont au minimum en classe IIa. Le guide européen MDCG 2019-11 (révisé en 2025) précise la frontière avec les applications de bien-être [3]. Pour le MVP, on reste sur la coordination, les rappels et le bien-être, **sans finalité médicale revendiquée**. Dépister un déclin cognitif ou interpréter des symptômes (V3) fera basculer vers le dispositif médical. |
| **Services à la personne (SAP)** | Si l'on veut que les clients bénéficient du crédit d'impôt, certaines prestations et la structure de l'entreprise doivent répondre aux conditions des services à la personne. À étudier en phase 0. |
| **Protection juridique des majeurs** | Pour une personne sous tutelle ou curatelle, ou avec des troubles cognitifs, le consentement et les droits d'accès de la famille doivent respecter le cadre légal. Sans mesure de protection, aucun proche ne peut consentir à la place du senior. Détails dans le [document 10](10-suivi-sante-a-distance.md). |
| **Secret professionnel et suivi par les proches** | Ce que les proches peuvent voir, ce que les professionnels peuvent leur transmettre, et ce qui relève de la télésurveillance médicale : voir le [document 10](10-suivi-sante-a-distance.md). |
| **Accessibilité** | Viser le niveau AA des règles d'accessibilité (WCAG / RGAA), et aller bien au-delà pour les seniors. |

## La charte éthique PPS (proposition)

Une charte publique, simple, compréhensible par une personne de 85 ans :

1. **Vous êtes chez vous.** C'est vous, le senior, qui êtes titulaire de votre
   compte. Vous décidez qui fait partie de votre cercle et ce que chacun
   peut voir.
2. **Vous savez qui voit quoi.** Un écran simple, lu à voix haute si besoin :
   « Sophie peut voir vos rendez-vous et vos prises de médicaments. Lucas
   peut voir les photos. »
3. **Pas de surveillance cachée.** Aucune caméra dans l'intimité, aucun suivi
   sans votre accord, aucune écoute en dehors des conversations avec votre
   compagnon.
4. **Vous parlez à une IA, et vous le savez.** Le compagnon ne se fait jamais
   passer pour un humain ni pour un membre de votre famille.
5. **L'IA ne remplace personne.** Elle est là pour vous rapprocher de vos
   proches, jamais pour les remplacer.
6. **Vos données ne sont pas à vendre.** Pas de publicité, pas de revente,
   pas d'utilisation pour entraîner des IA sans votre accord explicite.
7. **Droit à la déconnexion et à l'oubli.** Vous pouvez faire une pause,
   refuser un appel, effacer une conversation ou tout supprimer.
8. **Un humain en cas de doute.** Pour toute décision importante, un humain
   valide. Et vous pouvez toujours demander à parler à une vraie personne.

## La gouvernance éthique

- **Comité éthique** dès le début : seniors, aidants, gériatre, psychologue,
  juriste, représentant d'association. Il valide les fonctionnalités
  sensibles (géolocalisation, capteurs, détection de signaux faibles).
- **Revue périodique des consentements** : tous les 6 mois, ou à chaque
  changement de niveau d'accompagnement, on revérifie avec le senior ce qu'il
  accepte de partager.
- **Rapport de transparence annuel** : incidents, demandes d'accès,
  évolutions de l'IA.

## Les risques identifiés et leurs parades

| Risque | Parade |
|---|---|
| **Erreur de l'IA sur un médicament** | Validation humaine obligatoire de tout planning de traitement, l'IA ne modifie jamais un traitement, traçabilité complète. |
| **Alerte manquée** (pas de réaction alors que le senior a chuté) | Système d'alertes indépendant et redondant, mode dégradé sans IA, tests réguliers, partenariat téléassistance 24h/24. On ne promet jamais ce qu'on ne garantit pas. |
| **Fatigue d'alerte** (trop de fausses alertes, la famille n'y prête plus attention) | Seuils personnalisés, alertes expliquées, regroupement intelligent, mesure du taux de fausses alertes. |
| **Dépendance émotionnelle à l'IA** | L'IA pousse vers les proches, durée d'appel raisonnable, indicateur de contacts humains, revue par des psychologues. |
| **Sentiment d'être surveillé ou infantilisé** | Charte, transparence « qui voit quoi », le senior décide, ton respectueux (on vouvoie par défaut, on ne parle pas « bébé »). |
| **Conflits familiaux** (qui a accès, qui décide, qui paie) | Rôles clairs, décisions prises par le senior, équité visible de l'aide, cagnotte familiale transparente. |
| **Fuite de données** | Chiffrement, hébergement certifié, double authentification, audits de sécurité et tests d'intrusion réguliers. |
| **Usurpation par un arnaqueur** (quelqu'un se fait passer pour PPS) | Le compagnon a un nom et une voix connus, un mot de passe familial, et PPS ne demande **jamais** de code bancaire ni d'argent. Message répété régulièrement. |
| **Exclusion par le prix** | Offre gratuite utile, prise en charge par des financeurs (mutuelles, caisses, départements), tarif solidaire. |

## Sources

Relevées en octobre 2026 à partir de résultats de recherche web, à revérifier
avant diffusion externe.

1. Agence du numérique en santé, certification HDS :
   https://esante.gouv.fr/actualites/certification-hds-six-mois-pour-etre-pret
2. Goodwin, entrée en vigueur des obligations de transparence de l'AI Act
   (août 2026) :
   https://www.goodwinlaw.com/en/insights/publications/2026/08/alerts-technology-dpc-eu-ai-act-transparency-obligations-now-in-force
3. Guide MDCG 2019-11 sur la qualification des logiciels :
   https://openregulatory.com/mdcg/mdcg-2019-11
