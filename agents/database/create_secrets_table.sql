-- Secrets table for sensitive data storage
-- WARNING: Currently stores secrets in PLAINTEXT. FIDO2 encryption is planned but not implemented.
CREATE TABLE IF NOT EXISTS secrets (
    id VARCHAR PRIMARY KEY,
    frame_id VARCHAR,  -- Reference to frames table (nullable, cross-table ref)
    secret_type VARCHAR NOT NULL,
    raw_value BYTEA NOT NULL,  -- Unencrypted secret value (encryption planned)
    key_id VARCHAR,  -- Future: Which hardware key will be used for encryption (nullable until implemented)
    detected_at TIMESTAMPTZ NOT NULL,
    deployment_id VARCHAR NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_secrets_frame_id ON secrets(frame_id);
CREATE INDEX IF NOT EXISTS idx_secrets_deployment ON secrets(deployment_id);
CREATE INDEX IF NOT EXISTS idx_secrets_type ON secrets(secret_type);
CREATE INDEX IF NOT EXISTS idx_secrets_detected ON secrets(detected_at);
