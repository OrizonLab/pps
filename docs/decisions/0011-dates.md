# 0011. Dates et fuseaux horaires

- Statut : Acceptée
- Date : 2026-10-17

## Contexte

Les rappels suivent l'heure locale du senior, y compris lors des
changements d'heure et dans les fuseaux d'outre-mer.

## Décision

**jiff** (0.2.x), avec un pont vers SQLx pour les colonnes `timestamptz`.
Si le pont existant (`jiff-sqlx`) n'est pas compatible avec SQLx 0.9, un
newtype dans `pps-db` assure la conversion.

## Conséquences

- Un rappel se définit en **heure locale + fuseau du senior** (pas Paris en
  dur : outre-mer).
- Les occurrences sont calculées en UTC à l'avance, sans doublon possible.
- Heure inexistante (passage à l'heure d'été) : décalée après. Heure doublée
  (passage à l'heure d'hiver) : la première occurrence.
- Tests obligatoires aux dates de changement d'heure.
- Base des fuseaux embarquée dans le binaire.
