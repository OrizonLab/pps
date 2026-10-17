#!/bin/sh
# Exécuté par l'image officielle au premier démarrage (base vide) :
# crée les rôles de PPS à partir des secrets Docker.
set -eu
psql --no-psqlrc --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" \
    -v bdd="$POSTGRES_DB" \
    -v mdp_migration="$(cat /run/secrets/bdd_mdp_migration)" \
    -v mdp_api="$(cat /run/secrets/bdd_mdp_api)" \
    -f /opt/pps/roles.sql
