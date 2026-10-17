# 0013. Voix temps réel

- Statut : Acceptée
- Date : 2026-10-17

## Décision

Pas de prototype Python : tout en Rust, en trois temps.

1. Dès maintenant, en parallèle : un **banc d'essai de reconnaissance
   vocale** en ligne de commande, hors temps réel, sur des voix âgées
   enregistrées avec consentement en qualité téléphonique.
2. Incrément 7 : un **appel scripté sans IA** (voix de synthèse, réponses
   par oui/non ou par les touches), derrière un trait `CanalAppel`.
3. Après l'incrément 8 : l'agent conversationnel `pps-voix` (SDK Rust de
   LiveKit, détection d'activité vocale ONNX), branché sur le même trait.

## Conséquences

- Le vrai risque (compréhension des voix âgées) est mesuré tôt, sans temps
  réel.
- L'appel scripté est aussi le **mode dégradé** exigé de l'étage 1 quand
  l'IA est indisponible.
