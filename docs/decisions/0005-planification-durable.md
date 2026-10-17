# 0005. Planification durable des rappels et alertes

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

Un rappel de médicament ou une étape de cascade d'alerte ne doit pas être
perdu. Options étudiées : Temporal (SDK Rust stable depuis septembre 2026,
serveur en Go à exploiter), Restate, ou une file de tâches dans PostgreSQL.

## Décision

**File de tâches planifiées maison dans PostgreSQL**, derrière un trait
`Planificateur` (`programmer` et `annuler`, par clé d'idempotence).

- La tâche est créée **dans la même transaction** que l'acte métier (par
  exemple la validation d'un planning) : impossible d'avoir l'un sans
  l'autre.
- Réservation par `FOR UPDATE SKIP LOCKED` avec un **bail** ; la transaction
  de réservation est validée avant tout appel externe.
- Exécution **au moins une fois**, avec des effets idempotents (table des
  envois à contrainte d'unicité).
- Reprises avec délai croissant, file d'échecs qui alerte l'exploitant.
- Réveil par `LISTEN/NOTIFY`, avec scrutation périodique de secours.
- Les cascades d'alerte sont des machines à états enregistrées en base.
- Le contenu des tâches se limite à des identifiants (aucune donnée de
  santé).
- Horloge injectable (trait) pour des tests déterministes.

## Conséquences

- Un seul système avec état à sauvegarder et à faire héberger en HDS.
- Pas de pg_cron ni de pgmq (pas garantis sur un PostgreSQL managé HDS).
- Supervision : métrique du retard maximal des tâches échues, et sonde
  externe qui alerte si le planificateur se tait.
- Temporal ou Restate restent des options si les besoins dépassent la file.
