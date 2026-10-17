# 0006. Authentification

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

Les familles accèdent à des données de santé : l'authentification doit être
forte sans être compliquée, et le retrait d'un accès par le senior doit
prendre effet immédiatement.

## Décision

- **Famille** : lien magique par e-mail au MVP, **clés d'accès (passkeys)**
  avec webauthn-rs avant le pilote, **jamais de mot de passe**.
- Le **SMS** sert aux invitations (lien à usage unique) et aux notifications,
  pas à la connexion.
- **Sessions opaques stockées dans PostgreSQL**, pas de JWT : quand le senior
  retire l'accès d'un proche, la requête suivante doit être refusée.
- **Professionnels** hors MVP (Pro Santé Connect envisagé ensuite).
- **Le senior n'a pas de compte** : il est identifié par ses numéros,
  vérifiés lors de l'appel de bienvenue.

## Conséquences

- Jetons de 256 bits, stockés hachés, à usage unique, valables 15 minutes.
- Le lien ouvre une page de confirmation qui envoie un **POST** (les
  antivirus de messagerie suivent les liens).
- Réponse identique que le compte existe ou non, et limitation des essais.
- Cookie `__Host-`, HttpOnly, Secure, SameSite=Lax ; expiration à 30 jours,
  ou 7 jours sans activité ; contrôle de l'en-tête `Origin` et jeton CSRF.
- **Authentification récente obligatoire** pour produire une validation
  humaine (planning) ou modifier des droits.
- Le numéro d'un appelant peut être usurpé : un accord du senior ne se
  recueille que lors d'un appel **émis par PPS** ; un retrait d'accord est
  accepté par tout canal.
