#!/bin/sh
# Génère les secrets du développement local (mots de passe aléatoires) dans
# ./secrets/, ignoré par git. À lancer une fois avant `docker compose up`.
set -eu
cd "$(dirname "$0")/.."

mkdir -p secrets
# Le dossier n'est accessible qu'à l'utilisateur courant. Les fichiers sont
# lisibles par tous pour que les utilisateurs non root des conteneurs
# (postgres, nonroot) puissent les lire une fois montés.
chmod 700 secrets

for nom in bdd_mdp_admin bdd_mdp_migration bdd_mdp_api; do
    fichier="secrets/$nom.txt"
    if [ ! -s "$fichier" ]; then
        LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 32 >"$fichier"
        chmod 644 "$fichier"
    fi
done

echo "Secrets de développement prêts dans ./secrets (à ne jamais committer)."
