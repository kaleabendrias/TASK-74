-- ============================================================
-- facilities (referenced by users, lodgings, warehouses, etc.)
-- ============================================================
CREATE TABLE facilities (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          VARCHAR(255) NOT NULL,
    address       TEXT NOT NULL
);

-- ============================================================
-- users
-- ============================================================
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username        VARCHAR(150) NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    role            VARCHAR(50) NOT NULL DEFAULT 'Reviewer',
    facility_id     UUID REFERENCES facilities(id),
    totp_secret     BYTEA,
    mfa_enabled     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- sessions
-- ============================================================
CREATE TABLE sessions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash    TEXT NOT NULL UNIQUE,
    expires_at    TIMESTAMPTZ NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- csrf_tokens
-- ============================================================
CREATE TABLE csrf_tokens (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id    UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    token_hash    TEXT NOT NULL UNIQUE,
    expires_at    TIMESTAMPTZ NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- resources
-- ============================================================
CREATE TABLE resources (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title                 VARCHAR(500) NOT NULL,
    category              VARCHAR(255),
    tags                  JSONB NOT NULL DEFAULT '[]',
    hours                 JSONB NOT NULL DEFAULT '{}',
    pricing               JSONB NOT NULL DEFAULT '{}',
    contact_info_encrypted BYTEA,
    media_refs            JSONB NOT NULL DEFAULT '[]',
    address               TEXT,
    latitude              DOUBLE PRECISION,
    longitude             DOUBLE PRECISION,
    state                 VARCHAR(50) NOT NULL DEFAULT 'draft',
    scheduled_publish_at  TIMESTAMPTZ,
    current_version       INTEGER NOT NULL DEFAULT 1,
    created_by            UUID NOT NULL REFERENCES users(id),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- resource_versions
-- ============================================================
CREATE TABLE resource_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id     UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    version_number  INTEGER NOT NULL,
    snapshot        JSONB NOT NULL,
    changed_by      UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (resource_id, version_number)
);

-- ============================================================
-- lodgings
-- ============================================================
CREATE TABLE lodgings (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                  VARCHAR(500) NOT NULL,
    description           TEXT,
    state                 VARCHAR(50) NOT NULL DEFAULT 'draft',
    amenities             JSONB NOT NULL DEFAULT '[]',
    facility_id           UUID REFERENCES facilities(id),
    deposit_amount        NUMERIC(12, 2),
    monthly_rent          NUMERIC(12, 2),
    deposit_cap_validated BOOLEAN NOT NULL DEFAULT FALSE,
    created_by            UUID NOT NULL REFERENCES users(id),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- lodging_periods
-- ============================================================
CREATE TABLE lodging_periods (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lodging_id  UUID NOT NULL REFERENCES lodgings(id) ON DELETE CASCADE,
    start_date  DATE NOT NULL,
    end_date    DATE NOT NULL,
    min_nights  INTEGER NOT NULL DEFAULT 7,
    max_nights  INTEGER NOT NULL DEFAULT 365,
    vacancy     BOOLEAN NOT NULL DEFAULT TRUE
);

-- ============================================================
-- lodging_rent_changes
-- ============================================================
CREATE TABLE lodging_rent_changes (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lodging_id       UUID NOT NULL REFERENCES lodgings(id) ON DELETE CASCADE,
    proposed_rent    NUMERIC(12, 2) NOT NULL,
    proposed_deposit NUMERIC(12, 2) NOT NULL,
    status           VARCHAR(50) NOT NULL DEFAULT 'pending',
    requested_by     UUID NOT NULL REFERENCES users(id),
    reviewed_by      UUID REFERENCES users(id),
    reviewed_at      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- warehouses
-- ============================================================
CREATE TABLE warehouses (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    facility_id UUID NOT NULL REFERENCES facilities(id),
    name        VARCHAR(255) NOT NULL
);

-- ============================================================
-- bins
-- ============================================================
CREATE TABLE bins (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE CASCADE,
    label        VARCHAR(100) NOT NULL
);

-- ============================================================
-- inventory_lots
-- ============================================================
CREATE TABLE inventory_lots (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    facility_id       UUID NOT NULL REFERENCES facilities(id),
    warehouse_id      UUID NOT NULL REFERENCES warehouses(id),
    bin_id            UUID NOT NULL REFERENCES bins(id),
    item_name         VARCHAR(500) NOT NULL,
    lot_number        VARCHAR(255) NOT NULL,
    quantity_on_hand  INTEGER NOT NULL DEFAULT 0,
    quantity_reserved INTEGER NOT NULL DEFAULT 0,
    expiration_date   DATE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- inventory_transactions
-- ============================================================
CREATE TABLE inventory_transactions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lot_id       UUID NOT NULL REFERENCES inventory_lots(id),
    direction    VARCHAR(20) NOT NULL,
    quantity     INTEGER NOT NULL,
    reason       TEXT,
    performed_by UUID NOT NULL REFERENCES users(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_immutable BOOLEAN NOT NULL DEFAULT TRUE
);

-- ============================================================
-- media_files
-- ============================================================
CREATE TABLE media_files (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_name   VARCHAR(1000) NOT NULL,
    stored_path     TEXT NOT NULL,
    mime_type       VARCHAR(255) NOT NULL,
    size_bytes      BIGINT NOT NULL,
    checksum_sha256 VARCHAR(64) NOT NULL,
    uploaded_by     UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- review_decisions
-- ============================================================
CREATE TABLE review_decisions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL,
    entity_id   UUID NOT NULL,
    decision    VARCHAR(50) NOT NULL,
    comment     TEXT,
    decided_by  UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- import_jobs
-- ============================================================
CREATE TABLE import_jobs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type            VARCHAR(100) NOT NULL,
    file_path           TEXT NOT NULL,
    total_rows          INTEGER NOT NULL DEFAULT 0,
    processed_rows      INTEGER NOT NULL DEFAULT 0,
    progress_percent    SMALLINT NOT NULL DEFAULT 0,
    status              VARCHAR(50) NOT NULL DEFAULT 'queued',
    retries             INTEGER NOT NULL DEFAULT 0,
    max_retries         INTEGER NOT NULL DEFAULT 3,
    failure_log         TEXT,
    staging_table_name  VARCHAR(255),
    committed           BOOLEAN NOT NULL DEFAULT FALSE,
    created_by          UUID NOT NULL REFERENCES users(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- idempotency_keys
-- ============================================================
CREATE TABLE idempotency_keys (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_value   VARCHAR(500) NOT NULL UNIQUE,
    entity_type VARCHAR(100) NOT NULL,
    entity_id   UUID NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- api_connector_logs
-- ============================================================
CREATE TABLE api_connector_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    direction       VARCHAR(20) NOT NULL,
    endpoint        TEXT NOT NULL,
    nonce           VARCHAR(255),
    timestamp_sent  TIMESTAMPTZ,
    payload_hash    VARCHAR(128),
    status          VARCHAR(50) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- config_parameters
-- ============================================================
CREATE TABLE config_parameters (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile         VARCHAR(100) NOT NULL,
    key             VARCHAR(255) NOT NULL,
    value           TEXT NOT NULL,
    feature_switch  BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (profile, key)
);

-- ============================================================
-- audit_log
-- ============================================================
CREATE TABLE audit_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id    UUID REFERENCES users(id),
    action      VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id   UUID,
    detail      JSONB,
    ip_address  VARCHAR(45),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- export_approvals
-- ============================================================
CREATE TABLE export_approvals (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    export_type     VARCHAR(100) NOT NULL,
    requested_by    UUID NOT NULL REFERENCES users(id),
    approved_by     UUID REFERENCES users(id),
    watermark_text  TEXT,
    status          VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- Indexes for common lookups
-- ============================================================
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_resources_state ON resources(state);
CREATE INDEX idx_resources_created_by ON resources(created_by);
CREATE INDEX idx_lodgings_facility_id ON lodgings(facility_id);
CREATE INDEX idx_inventory_lots_facility ON inventory_lots(facility_id);
CREATE INDEX idx_inventory_transactions_lot ON inventory_transactions(lot_id);
CREATE INDEX idx_audit_log_actor ON audit_log(actor_id);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_import_jobs_status ON import_jobs(status);
