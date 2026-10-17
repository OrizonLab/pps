-- Rôles PostgreSQL de PPS (décision 0007), créés une fois par un
-- administrateur, avant la première migration.
--
-- Variables psql attendues :
--   bdd            nom de la base (ex. pps)
--   mdp_migration  mot de passe du rôle pps_migration
--   mdp_api        mot de passe du rôle pps_api
\set ON_ERROR_STOP on

-- Propriétaire de tous les objets : ne peut pas se connecter.
CREATE ROLE pps_proprietaire NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS;

-- Exécute les migrations en héritant des droits du propriétaire.
CREATE ROLE pps_migration LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS INHERIT
    PASSWORD :'mdp_migration';
GRANT pps_proprietaire TO pps_migration;

-- Rôle du service API : ne possède rien, ne contourne jamais la RLS.
CREATE ROLE pps_api LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS NOINHERIT
    PASSWORD :'mdp_api';

-- La base appartient au propriétaire ; seuls nos rôles peuvent s'y connecter.
ALTER DATABASE :"bdd" OWNER TO pps_proprietaire;
REVOKE ALL ON DATABASE :"bdd" FROM PUBLIC;
GRANT CONNECT ON DATABASE :"bdd" TO pps_migration, pps_api;
