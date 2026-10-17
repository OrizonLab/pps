# 0013. Voix temps réel

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

La compréhension des voix âgées au téléphone est le principal risque
technique, et tout notre code doit être en Rust (décision 0001).

## Décision

Pas de prototype Python : tout en Rust, en trois temps.

1. Dès maintenant, en parallèle : un **banc d'essai de reconnaissance
   vocale** en ligne de commande, hors temps réel, sur des voix âgées
   enregistrées avec consentement en qualité téléphonique.
2. Incrément 7 : un **appel scripté sans IA** (voix de synthèse, réponses
   par oui/non ou par les touches), derrière un trait `CanalAppel`.
3. Après l'incrément 8 : l'agent conversationnel `pps-voix` (SDK Rust de
   LiveKit, détection d'activité vocale ONNX), branché sur le même trait.

## Cadre du banc d'essai

Le banc d'essai est la **seule exception** à la règle « aucune donnée réelle
hors de la production HDS » (`CLAUDE.md`). Une voix reste une donnée
personnelle : il est donc strictement encadré.

- **Volontaires** majeurs, informés, consentants par écrit, pouvant retirer
  leur accord à tout moment.
- **Textes lus**, préparés à l'avance, **sans aucun contenu de santé** ni
  information personnelle.
- **Stockage chiffré dans l'EEE**, séparé de la base produit, accès limité à
  l'équipe du banc d'essai.
- Moteurs de reconnaissance testés sous **contrat**, sans réutilisation des
  enregistrements pour entraîner leurs modèles.
- **Suppression** des enregistrements à la fin du banc d'essai.

## Conséquences

- Le vrai risque (compréhension des voix âgées) est mesuré tôt, sans temps
  réel.
- L'appel scripté est aussi le **mode dégradé** exigé de l'étage 1 quand
  l'IA est indisponible.
