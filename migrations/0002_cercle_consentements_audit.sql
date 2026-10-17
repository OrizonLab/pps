-- Cercle de soin, membres, consentements et journal d'audit (incrément 2).
--
-- Isolation (décision 0007) : chaque table liée à un cercle a la
-- Row-Level Security activée et forcée. Le contexte est fixé par
-- transaction avec set_config(..., true) :
--   app.cercle_id      le cercle de la transaction ;
--   app.utilisateur_id l'utilisateur connecté, s'il y en a un.
-- Sans contexte, aucune ligne n'est visible.

SET LOCAL ROLE pps_proprietaire;

-- Contexte de la transaction. Une valeur invalide fait échouer la requête
-- (plutôt que de tout montrer).
CREATE FUNCTION app.contexte_cercle() RETURNS uuid
    LANGUAGE sql STABLE
    RETURN NULLIF(current_setting('app.cercle_id', true), '')::uuid;

CREATE FUNCTION app.contexte_utilisateur() RETURNS uuid
    LANGUAGE sql STABLE
    RETURN NULLIF(current_setting('app.utilisateur_id', true), '')::uuid;

GRANT EXECUTE ON FUNCTION app.contexte_cercle(), app.contexte_utilisateur() TO pps_api;

-- Les tables historiques n'acceptent ni modification ni suppression, même
-- par leur propriétaire.
CREATE FUNCTION app.interdire_modification() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'la table %.% n''accepte que des insertions', TG_TABLE_SCHEMA, TG_TABLE_NAME
        USING ERRCODE = 'insufficient_privilege';
END;
$$;

-- Compte de l'espace famille. Les informations de connexion arriveront
-- avec l'authentification (incrément 3).
CREATE TABLE app.utilisateur (
    id uuid PRIMARY KEY,
    cree_le timestamptz NOT NULL
);

ALTER TABLE app.utilisateur ENABLE ROW LEVEL SECURITY;
ALTER TABLE app.utilisateur FORCE ROW LEVEL SECURITY;
CREATE POLICY utilisateur_lui_meme ON app.utilisateur
    USING (id = app.contexte_utilisateur())
    WITH CHECK (id = app.contexte_utilisateur());

-- Le cercle de soin, avec l'identité du senior (sans aucune donnée de santé).
CREATE TABLE app.cercle (
    id uuid PRIMARY KEY,
    statut text NOT NULL
        CHECK (statut IN ('en_preparation', 'actif', 'refuse', 'clos')),
    senior_prenom text NOT NULL CHECK (char_length(senior_prenom) BETWEEN 1 AND 100),
    senior_appellation text NOT NULL CHECK (char_length(senior_appellation) BETWEEN 1 AND 100),
    senior_vouvoiement boolean NOT NULL,
    fuseau_horaire text NOT NULL CHECK (char_length(fuseau_horaire) BETWEEN 1 AND 64),
    cree_le timestamptz NOT NULL,
    modifie_le timestamptz NOT NULL
);

ALTER TABLE app.cercle ENABLE ROW LEVEL SECURITY;
ALTER TABLE app.cercle FORCE ROW LEVEL SECURITY;
CREATE POLICY cercle_du_contexte ON app.cercle
    USING (id = app.contexte_cercle())
    WITH CHECK (id = app.contexte_cercle());

-- Appartenance d'un utilisateur à un cercle.
CREATE TABLE app.membre (
    id uuid PRIMARY KEY,
    cercle_id uuid NOT NULL REFERENCES app.cercle (id),
    utilisateur_id uuid NOT NULL REFERENCES app.utilisateur (id),
    role text NOT NULL CHECK (role IN ('organisateur', 'proche')),
    lien text NOT NULL
        CHECK (lien IN ('enfant', 'petit_enfant', 'conjoint', 'fratrie', 'ami', 'voisin', 'autre')),
    nom_affiche text NOT NULL CHECK (char_length(nom_affiche) BETWEEN 1 AND 100),
    ajoute_le timestamptz NOT NULL,
    retire_le timestamptz CHECK (retire_le IS NULL OR retire_le >= ajoute_le),
    UNIQUE (cercle_id, id)
);

-- Un utilisateur n'est membre actif qu'une fois par cercle.
CREATE UNIQUE INDEX membre_actif_unique ON app.membre (cercle_id, utilisateur_id)
    WHERE retire_le IS NULL;
CREATE INDEX membre_par_utilisateur ON app.membre (utilisateur_id);

ALTER TABLE app.membre ENABLE ROW LEVEL SECURITY;
ALTER TABLE app.membre FORCE ROW LEVEL SECURITY;
CREATE POLICY membre_du_cercle ON app.membre
    USING (cercle_id = app.contexte_cercle())
    WITH CHECK (cercle_id = app.contexte_cercle());

-- Historique des accords et retraits (en insertion seule).
CREATE TABLE app.consentement_evenement (
    id uuid PRIMARY KEY,
    cercle_id uuid NOT NULL,
    membre_id uuid NOT NULL,
    categorie text NOT NULL
        CHECK (categorie IN ('prises_medicaments', 'rendez_vous', 'bien_etre', 'check_in', 'alertes')),
    nature text NOT NULL CHECK (nature IN ('accord', 'retrait')),
    origine text NOT NULL
        CHECK (origine IN ('appel_sortant', 'appel_entrant', 'conseiller', 'renonciation_membre')),
    -- Appel concerné ; la table des appels arrivera à l'incrément 7.
    id_appel uuid,
    survenu_le timestamptz NOT NULL,
    enregistre_le timestamptz NOT NULL DEFAULT now(),
    -- Un accord ne se recueille que lors d'un appel émis par PPS (décision 0006).
    CONSTRAINT accord_par_appel_sortant CHECK (nature = 'retrait' OR origine = 'appel_sortant'),
    CONSTRAINT appel_renseigne_si_appel CHECK (
        (origine IN ('appel_sortant', 'appel_entrant')) = (id_appel IS NOT NULL)
    ),
    -- Le membre appartient au même cercle.
    FOREIGN KEY (cercle_id, membre_id) REFERENCES app.membre (cercle_id, id)
);

CREATE INDEX consentement_par_membre
    ON app.consentement_evenement (cercle_id, membre_id, categorie, survenu_le);

ALTER TABLE app.consentement_evenement ENABLE ROW LEVEL SECURITY;
ALTER TABLE app.consentement_evenement FORCE ROW LEVEL SECURITY;
CREATE POLICY consentement_du_cercle ON app.consentement_evenement
    USING (cercle_id = app.contexte_cercle())
    WITH CHECK (cercle_id = app.contexte_cercle());

REVOKE UPDATE, DELETE ON app.consentement_evenement FROM pps_api;
CREATE TRIGGER consentement_insertion_seule
    BEFORE UPDATE OR DELETE ON app.consentement_evenement
    FOR EACH STATEMENT EXECUTE FUNCTION app.interdire_modification();

-- Journal d'audit, chaîné cercle par cercle (en insertion seule).
CREATE TABLE app.audit (
    cercle_id uuid NOT NULL REFERENCES app.cercle (id),
    sequence bigint NOT NULL CHECK (sequence >= 1),
    acteur_type text NOT NULL CHECK (acteur_type IN ('utilisateur', 'senior', 'systeme')),
    acteur_id uuid,
    action text NOT NULL CHECK (char_length(action) BETWEEN 1 AND 64),
    objet_type text NOT NULL CHECK (char_length(objet_type) BETWEEN 1 AND 64),
    objet_id uuid NOT NULL,
    survenu_le timestamptz NOT NULL,
    empreinte_precedente bytea NOT NULL CHECK (octet_length(empreinte_precedente) = 32),
    empreinte bytea NOT NULL CHECK (octet_length(empreinte) = 32),
    PRIMARY KEY (cercle_id, sequence),
    CHECK ((acteur_type = 'utilisateur') = (acteur_id IS NOT NULL))
);

ALTER TABLE app.audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE app.audit FORCE ROW LEVEL SECURITY;
CREATE POLICY audit_du_cercle ON app.audit
    USING (cercle_id = app.contexte_cercle())
    WITH CHECK (cercle_id = app.contexte_cercle());

REVOKE UPDATE, DELETE ON app.audit FROM pps_api;
CREATE TRIGGER audit_insertion_seule
    BEFORE UPDATE OR DELETE ON app.audit
    FOR EACH STATEMENT EXECUTE FUNCTION app.interdire_modification();

RESET ROLE;
