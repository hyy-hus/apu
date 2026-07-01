CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    username        TEXT NOT NULL,
    email           TEXT NOT NULL,
    name            TEXT NOT NULL,
    password_hash   TEXT NOT NULL,

    created_at      TIMESTAMPTZ DEFAULT now(),
    updated_at      TIMESTAMPTZ DEFAULT now(),
    deleted_at      TIMESTAMPTZ DEFAULT NULL
);

CREATE UNIQUE INDEX users_active_email_idx ON users (email) WHERE deleted_at IS NULL
CREATE UNIQUE INDEX users_active_username_idx ON users (username) WHERE deleted_at IS NULL;


