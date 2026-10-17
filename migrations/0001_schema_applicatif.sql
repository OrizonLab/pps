-- Socle : schéma applicatif et droits (décision 0007).
--
-- Les rôles sont créés par l'infrastructure avant toute migration
-- (voir docker/postgres/roles.sql) :
--   pps_proprietaire : propriétaire de tous les objets, sans connexion ;
--   pps_migration    : exécute les migrations, membre de pps_proprietaire ;
--   pps_api          : rôle du service API, sans BYPASSRLS, ne possède rien.
--
-- Conventions vérifiées par des tests :
-- - chaque migration fixe le rôle pps_proprietaire (pour que les objets lui
--   appartiennent) et le rétablit à la fin (SQLx enregistre ensuite la
--   migration avec le rôle de connexion) ;
-- - aucun droit n'est accordé par défaut : chaque migration accorde
--   explicitement, table par table, les droits strictement nécessaires.
--   Un oubli fait échouer un test au lieu d'ouvrir des droits sans bruit.

-- Le schéma est créé avec le rôle de connexion, qui a le droit de créer
-- dans la base (propriétaire ou membre du propriétaire).
CREATE SCHEMA app AUTHORIZATION pps_proprietaire;

SET LOCAL ROLE pps_proprietaire;

REVOKE ALL ON SCHEMA app FROM PUBLIC;
GRANT USAGE ON SCHEMA app TO pps_api;

-- Le droit d'exécuter les fonctions accordé à PUBLIC est un défaut global :
-- il se retire sans clause IN SCHEMA (une clause de schéma ne retirerait
-- que ce qu'une clause de schéma a ajouté).
ALTER DEFAULT PRIVILEGES FOR ROLE pps_proprietaire
    REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;

RESET ROLE;
