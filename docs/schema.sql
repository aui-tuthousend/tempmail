CREATE TABLE accounts (
    id              UUID PRIMARY KEY,

    username        VARCHAR(64) NOT NULL UNIQUE,
    local_part      VARCHAR(64) NOT NULL,
    domain          VARCHAR(255) NOT NULL,
    address         VARCHAR(320) GENERATED ALWAYS AS (local_part || '@' || domain) STORED,

    password_hash   VARCHAR(255) NOT NULL,
    display_name    VARCHAR(100),
    is_active       BOOLEAN NOT NULL DEFAULT true,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (local_part, domain)
);

CREATE INDEX idx_accounts_address ON accounts(address);

CREATE TABLE sessions (
    id              UUID PRIMARY KEY,

    token_hash      VARCHAR(255) NOT NULL UNIQUE,
    device_info     JSONB,
    expires_at      TIMESTAMPTZ NOT NULL,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_token_hash ON sessions(token_hash);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

CREATE TABLE session_accounts (
    id              UUID PRIMARY KEY,
    session_id      UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    account_id      UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

    is_active       BOOLEAN NOT NULL DEFAULT false,
    logged_in_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (session_id, account_id)
);

CREATE INDEX idx_session_accounts_session ON session_accounts(session_id);
CREATE INDEX idx_session_accounts_account ON session_accounts(account_id);

CREATE UNIQUE INDEX idx_one_active_account_per_session
ON session_accounts(session_id)
WHERE is_active = true;

CREATE TABLE api_keys (
    id              UUID PRIMARY KEY,

    name            VARCHAR(100),
    key_hash        VARCHAR(255) NOT NULL UNIQUE,
    key_prefix      VARCHAR(16) NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    last_used_at    TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_active ON api_keys(is_active);
CREATE INDEX idx_api_keys_expires_at ON api_keys(expires_at) WHERE expires_at IS NOT NULL;

CREATE TABLE messages (
    id              UUID PRIMARY KEY,
    account_id      UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

    message_id      VARCHAR(998),
    in_reply_to     VARCHAR(998),

    from_address    VARCHAR(320) NOT NULL,
    from_name       VARCHAR(255),
    to_addresses    JSONB NOT NULL,
    cc_addresses    JSONB,
    bcc_addresses   JSONB,

    subject         TEXT,
    text_body       TEXT,
    html_body       TEXT,

    is_read         BOOLEAN NOT NULL DEFAULT false,
    is_starred      BOOLEAN NOT NULL DEFAULT false,
    is_archived     BOOLEAN NOT NULL DEFAULT false,
    is_deleted      BOOLEAN NOT NULL DEFAULT false,

    has_attachments BOOLEAN NOT NULL DEFAULT false,
    size_bytes      INTEGER NOT NULL DEFAULT 0,

    received_at     TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_account_id ON messages(account_id);
CREATE INDEX idx_messages_flags ON messages(account_id, is_archived, is_deleted, is_starred);
CREATE INDEX idx_messages_received_at ON messages(account_id, received_at DESC);

CREATE TABLE attachments (
    id              UUID PRIMARY KEY,
    message_id      UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,

    filename        VARCHAR(255) NOT NULL,
    content_type    VARCHAR(127),
    size_bytes      INTEGER NOT NULL,
    storage_key     VARCHAR(512) NOT NULL,
    content_id      VARCHAR(255),

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attachments_message_id ON attachments(message_id);
