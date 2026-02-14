-- Core schema for recall-pipeline
-- Requires: TimescaleDB, pgvector

-- CREATE EXTENSION IF NOT EXISTS timescaledb;
-- CREATE EXTENSION IF NOT EXISTS vector;

-- Frames: the fundamental capture unit
CREATE TABLE frames (
    id              UUID NOT NULL,
    captured_at     TIMESTAMPTZ NOT NULL,
    deployment_id   TEXT,
    image_ref       TEXT NOT NULL,
    image_size_bytes BIGINT,
    phash           BIGINT NOT NULL,
    phash_prefix    SMALLINT NOT NULL,
    has_text        BOOLEAN DEFAULT FALSE,
    has_activity    BOOLEAN DEFAULT FALSE,
    window_title    TEXT,
    app_name        TEXT,
    ocr_text        TEXT,
    vision_summary  TEXT,
    vision_status   SMALLINT DEFAULT 0, -- 0=pending, 1=processed, 2=failed, 3=skipped
    embedding       REAL[], -- STUBBED (was VECTOR(384))
    embedding_status SMALLINT DEFAULT 0,
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (id, captured_at)
);

-- SELECT create_hypertable('frames', 'captured_at');

CREATE INDEX idx_frames_phash_prefix ON frames (phash_prefix);
CREATE INDEX idx_frames_deployment ON frames (deployment_id);
CREATE INDEX idx_frames_app ON frames (app_name);
CREATE INDEX idx_frames_ocr_fts ON frames USING GIN (to_tsvector('english', coalesce(ocr_text, '')));

-- OCR text (separate table for multiple OCR passes / bounding boxes)
CREATE TABLE ocr_text (
    id          BIGSERIAL PRIMARY KEY,
    frame_id    UUID NOT NULL,
    text        TEXT,
    confidence  REAL,
    language    TEXT,
    bbox        JSONB,
    created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_ocr_frame_id ON ocr_text(frame_id);
CREATE INDEX idx_ocr_text_fts ON ocr_text USING GIN (to_tsvector('english', coalesce(text, '')));

-- Window / app context
CREATE TABLE window_context (
    id              BIGSERIAL PRIMARY KEY,
    frame_id        UUID NOT NULL,
    app_name        TEXT,
    window_title    TEXT,
    process_name    TEXT,
    is_focused      BOOLEAN,
    url             TEXT,
    created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_wc_frame_id ON window_context(frame_id);
CREATE INDEX idx_wc_app ON window_context(app_name);

-- Activity metrics (hypertable)
CREATE TABLE activity (
    id              BIGSERIAL NOT NULL,
    timestamp       TIMESTAMPTZ NOT NULL,
    activity_type   TEXT,
    keystroke_count  INTEGER,
    mouse_events    INTEGER,
    idle_seconds    INTEGER,
    PRIMARY KEY (id, timestamp)
);

-- SELECT create_hypertable('activity', 'timestamp', if_not_exists => TRUE);

-- Sessions
CREATE TABLE sessions (
    id              UUID PRIMARY KEY,
    start_time      TIMESTAMPTZ NOT NULL,
    end_time        TIMESTAMPTZ NOT NULL,
    primary_app     TEXT,
    frame_count     INTEGER,
    total_ocr_words INTEGER,
    summary         TEXT,
    created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_sessions_times ON sessions(start_time, end_time);
