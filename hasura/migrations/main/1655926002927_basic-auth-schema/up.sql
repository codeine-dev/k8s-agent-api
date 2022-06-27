CREATE SCHEMA "auth";

CREATE TABLE "auth"."actor" (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE "auth"."role" (
    value TEXT PRIMARY KEY NOT NULL
);

CREATE TABLE "auth"."account" (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    actor_id uuid NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT "fk_account_actor" FOREIGN KEY (actor_id) REFERENCES "auth"."actor" (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE "auth"."account_has_roles" (
    account_id uuid NOT NULL,
    role_value TEXT NOT NULL,
    CONSTRAINT "pk_account_has_roles" PRIMARY KEY (account_id, role_value),
    CONSTRAINT "fk_account_has_roles_account" FOREIGN KEY (account_id) REFERENCES "auth"."account" (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_account_has_roles_role" FOREIGN KEY (role_value) REFERENCES "auth"."role" (value) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE "auth"."access_token" (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    token TEXT NOT NULL,
    account_id uuid NOT NULL,
    overwrite_actor_id uuid,
    has_own_roles bool NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT "fk_access_token_account" FOREIGN KEY (account_id) REFERENCES "auth"."account" (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_access_token_actor" FOREIGN KEY (overwrite_actor_id) REFERENCES "auth"."actor" (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE "auth"."access_token_has_roles" (
    access_token_id uuid NOT NULL,
    role_value TEXT NOT NULL,
    CONSTRAINT "pk_access_token_has_roles" PRIMARY KEY (access_token_id, role_value),
    CONSTRAINT "fk_access_token_has_roles_account" FOREIGN KEY (access_token_id) REFERENCES "auth"."access_token" (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_access_token_has_roles_role" FOREIGN KEY (role_value) REFERENCES "auth"."role" (value) ON DELETE CASCADE ON UPDATE CASCADE
);
