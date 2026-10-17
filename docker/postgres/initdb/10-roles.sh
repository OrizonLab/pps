#!/bin/sh
# Exécuté par l'image officielle au premier démarrage (base vide) :
# crée les rôles de PPS à partir des secrets Docker. Les mots de passe sont
# lus par psql lui-même, jamais passés en argument (visibles dans `ps`).
set -eu
psql --no-psqlrc --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" \
    -v bdd="$POSTGRES_DB" <<'EOSQL'
\set mdp_migration `cat /run/secrets/bdd_mdp_migration`
\set mdp_api `cat /run/secrets/bdd_mdp_api`
\i /opt/pps/roles.sql
EOSQL
