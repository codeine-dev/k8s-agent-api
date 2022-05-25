CREATE TABLE IF NOT EXISTS "user" (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "organization" (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name Text NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "user_belongs_to_organization" (
    user_id uuid NOT NULL,
    organization_id uuid NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT "pk_user_belongs_to_organization" PRIMARY KEY (user_id, organization_id),
    CONSTRAINT "fk_user_belongs_to_organization_user" FOREIGN KEY (user_id) REFERENCES "user" (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_user_belongs_to_organization_organization" FOREIGN KEY (organization_id) REFERENCES "organization" (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS "organization_registration_token" (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT "uk_organization_registration_token" UNIQUE (name, organization_id),
    CONSTRAINT "fk_organization_registration_token_organization" FOREIGN KEY (organization_id) REFERENCES "organization" (id) ON DELETE CASCADE ON UPDATE CASCADE
);


CREATE TABLE IF NOT EXISTS "cluster" (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name Text NOT NULL DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "cluster_registered_with_token" (
    cluster_id uuid PRIMARY KEY NOT NULL,
    token_id uuid,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT "uk_cluster_registered_with_token" UNIQUE (cluster_id, token_id),
    CONSTRAINT "fk_cluster_registered_with_token_cluster" FOREIGN KEY (cluster_id) REFERENCES "cluster" (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_cluster_registered_with_token_token" FOREIGN KEY (token_id) REFERENCES "organization_registration_token" (id) ON DELETE SET NULL ON UPDATE SET NULL
);

CREATE TABLE IF NOT EXISTS "cluster_belongs_to_organization" (
    cluster_id uuid NOT NULL,
    organization_id uuid NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT "pk_cluster_belongs_to_organization" PRIMARY KEY (cluster_id, organization_id),
    CONSTRAINT "fk_cluster_belongs_to_organization_cluster" FOREIGN KEY (cluster_id) REFERENCES "cluster" (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_cluster_belongs_to_organization_organization" FOREIGN KEY (organization_id) REFERENCES "organization" (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE OR REPLACE FUNCTION "generate_token_value"(organization_id uuid, name TEXT, hasura_session json) RETURNS "organization_registration_token" AS $$
DECLARE
    plain bytea;
    plainencoded TEXT;
    hashed TEXT;
    row "organization_registration_token"%ROWTYPE;
BEGIN
    -- TODO: validate access to organization_id usin hasura_session
    plain := gen_random_bytes(32);
    plainencoded := encode(plain, 'base64');
    hashed := encode(digest(plain, 'sha256'), 'base64');

    INSERT INTO "organization_registration_token" ("organization_id", "name", "value") VALUES (organization_id, name, hashed) RETURNING * INTO row;

    row.value = plainencoded;
    RETURN row;
END
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION "register"(token TEXT) RETURNS "cluster" AS $$
DECLARE
    row "cluster"%ROWTYPE;
    token_id uuid;
    org_id uuid;
    hashed TEXT;
BEGIN
    hashed := encode(digest(decode(token, 'base64'), 'sha256'), 'base64');
    SELECT id, organization_id FROM organization_registration_token WHERE value = hashed INTO token_id, org_id;

    if org_id is null OR token_id is null then
        RAISE EXCEPTION 'INVALID REGISTRATION TOKEN' USING HINT = 'Please check your registration token';
    end if;

    INSERT INTO "cluster" DEFAULT VALUES RETURNING * INTO row;

    INSERT INTO "cluster_registered_with_token" (cluster_id, token_id) VALUES (row.id, token_id);
    INSERT INTO "cluster_belongs_to_organization" (cluster_id, organization_id) VALUES (row.id, org_id);

    RETURN row;
END;
$$ LANGUAGE plpgsql;