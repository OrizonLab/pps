# 0010. Langue du code

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

Le porteur, la documentation et les textes juridiques sont en français, et
certains termes métier (aidant, personne de confiance) se traduisent mal.

## Décision

**Ce que nous nommons est en français ; ce que Rust et les bibliothèques
imposent reste tel quel.**

- Identifiants, tables (au singulier), colonnes et routes : français en
  ASCII, sans accents (`PlanningValide`, `consentement.cree_le`), avec
  `non_ascii_idents` interdit.
- Commentaires, documentation et messages d'erreur : français accentué.
- Messages de commit : format Conventional Commits, type en anglais,
  description en français (`feat(soin): cascade d'alerte`).

## Conséquences

Le code parle le même vocabulaire que la documentation, le porteur et les
textes juridiques (aidant, prise, consentement, personne de confiance).
