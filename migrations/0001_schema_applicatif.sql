-- Socle : schéma applicatif et droits (décision 0007).
--
-- Les rôles sont créés par l'infrastructure avant toute migration
-- (voir docker/postgres/roles.sql) :
--   pps_proprietaire : propriétaire de tous les objets, sans connexion ;
--   pps_migration    : exécute les migrations, membre de pps_proprietaire ;
--   pps_api          : rôle du service API, sans BYPASSRLS, ne possède rien.
--
-- Convention vérifiée par un test : chaque migration fixe le rôle
-- pps_proprietaire (pour que les objets lui appartiennent) et le rétablit
-- à la fin (SQLx enregistre ensuite la migration avec le rôle de connexion).

-- Le schéma est créé avec le rôle de connexion, qui a le droit de créer
-- dans la base (propriétaire ou membre du propriétaire).
CREATE SCHEMA app AUTHORIZATION pps_proprietaire;

SET LOCAL ROLE pps_proprietaire;

REVOKE ALL ON SCHEMA app FROM PUBLIC;
GRANT USAGE ON SCHEMA app TO pps_api;

ALTER DEFAULT PRIVILEGES FOR ROLE pps_proprietaire IN SCHEMA app
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO pps_api;
ALTER DEFAULT PRIVILEGES FOR ROLE pps_proprietaire IN SCHEMA app
    GRANT USAGE, SELECT ON SEQUENCES TO pps_api;
ALTER DEFAULT PRIVILEGES FOR ROLE pps_proprietaire IN SCHEMA app
    REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;

RESET ROLE;
