-- PostgreSQL Schema Upgrade Script
-- Adds TimescaleDB and pgvector extensions to existing recall database
-- Date: 2025-11-24

BEGIN;

-- ===================================================================
-- STEP 1: Enable Extensions
-- ===================================================================

-- Enable TimescaleDB
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Enable pgvector
CREATE EXTENSION IF NOT EXISTS vector;

-- Verify extensions
DO $$
DECLARE
    timescaledb_version TEXT;
    pgvector_version TEXT;
BEGIN
    SELECT extversion INTO timescaledb_version FROM pg_extension WHERE extname = 'timescaledb';
    SELECT extversion INTO pgvector_version FROM pg_extension WHERE extname = 'vector';

    IF timescaledb_version IS NULL THEN
        RAISE EXCEPTION 'TimescaleDB extension not installed';
    END IF;

    IF pgvector_version IS NULL THEN
        RAISE EXCEPTION 'pgvector extension not installed';
    END IF;

    RAISE NOTICE '✓ TimescaleDB version: %', timescaledb_version;
    RAISE NOTICE '✓ pgvector version: %', pgvector_version;
END;
$$;

-- ===================================================================
-- STEP 2: Create or Alter frames Table
-- ===================================================================

-- Check if frames table exists
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'frames') THEN
        RAISE NOTICE '✓ frames table exists, will upgrade in place';
    ELSE
        RAISE NOTICE '✓ frames table does not exist, will create new';
    END IF;
END;
$$;

-- Create frames table if it doesn't exist
CREATE TABLE IF NOT EXISTS frames (
    id UUID NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL,
    window_title TEXT,
    app_name TEXT,
    image_ref TEXT,
    phash BIGINT,
    phash_prefix SMALLINT,
    ocr_text TEXT,
    ocr_status SMALLINT DEFAULT 0,
    embedding VECTOR(384),  -- pgvector type
    embedding_status SMALLINT DEFAULT 0,
    vision_summary TEXT,
    vision_status SMALLINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT now(),
    deployment_id TEXT,
    image_size_bytes INTEGER,
    has_text BOOLEAN DEFAULT FALSE,
    has_activity BOOLEAN DEFAULT FALSE,
    activity_id UUID,
    PRIMARY KEY (id, captured_at)
);

-- Add missing columns if frames table already exists
DO $$
BEGIN
    -- Add deployment_id if missing
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'frames' AND column_name = 'deployment_id') THEN
        ALTER TABLE frames ADD COLUMN deployment_id TEXT;
        RAISE NOTICE '✓ Added deployment_id column';
    END IF;

    -- Add image_size_bytes if missing
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'frames' AND column_name = 'image_size_bytes') THEN
        ALTER TABLE frames ADD COLUMN image_size_bytes INTEGER;
        RAISE NOTICE '✓ Added image_size_bytes column';
    END IF;

    -- Add has_text if missing
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'frames' AND column_name = 'has_text') THEN
        ALTER TABLE frames ADD COLUMN has_text BOOLEAN DEFAULT FALSE;
        RAISE NOTICE '✓ Added has_text column';
    END IF;

    -- Add has_activity if missing
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'frames' AND column_name = 'has_activity') THEN
        ALTER TABLE frames ADD COLUMN has_activity BOOLEAN DEFAULT FALSE;
        RAISE NOTICE '✓ Added has_activity column';
    END IF;

    -- Add embedding if missing (this is critical!)
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'frames' AND column_name = 'embedding') THEN
        ALTER TABLE frames ADD COLUMN embedding VECTOR(384);
        RAISE NOTICE '✓ Added embedding column';
    END IF;

    -- Add embedding_status if missing
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'frames' AND column_name = 'embedding_status') THEN
        ALTER TABLE frames ADD COLUMN embedding_status SMALLINT DEFAULT 0;
        RAISE NOTICE '✓ Added embedding_status column';
    END IF;
    
    -- Add activity_id if missing
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'frames' AND column_name = 'activity_id') THEN
        ALTER TABLE frames ADD COLUMN activity_id UUID;
        RAISE NOTICE '✓ Added activity_id column';
    END IF;
END;
$$;

-- Add unique constraint on id for foreign key references
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_type = 'UNIQUE'
          AND table_name = 'frames'
          AND constraint_name = 'frames_id_unique'
    ) THEN
        ALTER TABLE frames ADD CONSTRAINT frames_id_unique UNIQUE (id);
        RAISE NOTICE '✓ Added frames_id_unique constraint';
    END IF;
END;
$$;

-- ===================================================================
-- STEP 3: Convert frames to Hypertable
-- ===================================================================

-- Check if frames is already a hypertable
DO $$
DECLARE
    is_hypertable BOOLEAN;
BEGIN
    SELECT EXISTS (
        SELECT 1 FROM timescaledb_information.hypertables
        WHERE hypertable_name = 'frames'
    ) INTO is_hypertable;

    IF is_hypertable THEN
        RAISE NOTICE '✓ frames is already a hypertable';
    ELSE
        -- Convert to hypertable
        PERFORM create_hypertable('frames', 'captured_at',
                                  if_not_exists => TRUE,
                                  migrate_data => TRUE);
        RAISE NOTICE '✓ Converted frames to hypertable';
    END IF;
END;
$$;

-- ===================================================================
-- STEP 4: Create Indexes
-- ===================================================================

-- Phash prefix index
CREATE INDEX IF NOT EXISTS idx_frames_phash_prefix ON frames (phash_prefix);

-- Full-text search on OCR text (GIN index)
CREATE INDEX IF NOT EXISTS idx_frames_ocr_text_gin
ON frames USING GIN (to_tsvector('english', coalesce(ocr_text, '')));

-- Timestamp index
CREATE INDEX IF NOT EXISTS idx_frames_created_at ON frames (created_at);

-- App name index
CREATE INDEX IF NOT EXISTS idx_frames_app_name ON frames (app_name);

-- Device name index
CREATE INDEX IF NOT EXISTS idx_frames_deployment_id ON frames (deployment_id);

-- Vector similarity index (HNSW for fast ANN search)
-- Note: Only create if embeddings exist
DO $$
DECLARE
    embedding_count BIGINT;
BEGIN
    SELECT COUNT(*) INTO embedding_count FROM frames WHERE embedding IS NOT NULL;

    IF embedding_count > 0 THEN
        -- Create HNSW index for vector similarity search
        CREATE INDEX IF NOT EXISTS idx_frames_embedding_hnsw
        ON frames USING hnsw (embedding vector_cosine_ops);
        RAISE NOTICE '✓ Created HNSW vector index (% embeddings)', embedding_count;
    ELSE
        RAISE NOTICE '⚠ No embeddings found, skipping vector index creation';
    END IF;
END;
$$;

-- ===================================================================
-- STEP 5: Create Related Tables
-- ===================================================================

-- OCR text table
CREATE TABLE IF NOT EXISTS ocr_text (
    id BIGSERIAL PRIMARY KEY,
    frame_id UUID NOT NULL REFERENCES frames(id) ON DELETE CASCADE,
    text TEXT,
    confidence REAL,
    language TEXT,
    bbox JSONB,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ocr_frame_id ON ocr_text(frame_id);
CREATE INDEX IF NOT EXISTS idx_ocr_text_fts ON ocr_text
USING GIN (to_tsvector('english', coalesce(text, '')));

-- Window context table
CREATE TABLE IF NOT EXISTS window_context (
    id BIGSERIAL PRIMARY KEY,
    frame_id UUID NOT NULL REFERENCES frames(id) ON DELETE CASCADE,
    app_name TEXT,
    window_title TEXT,
    process_name TEXT,
    is_focused BOOLEAN,
    url TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_wc_frame_id ON window_context(frame_id);
CREATE INDEX IF NOT EXISTS idx_wc_app ON window_context(app_name);
CREATE INDEX IF NOT EXISTS idx_wc_focused ON window_context(is_focused);

-- Activity metrics table
CREATE TABLE IF NOT EXISTS activity (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    activity_type TEXT,
    keystroke_count INTEGER,
    mouse_events INTEGER,
    idle_seconds INTEGER
);

-- Convert activity to hypertable
DO $$
DECLARE
    is_hypertable BOOLEAN;
BEGIN
    SELECT EXISTS (
        SELECT 1 FROM timescaledb_information.hypertables
        WHERE hypertable_name = 'activity'
    ) INTO is_hypertable;

    IF NOT is_hypertable THEN
        PERFORM create_hypertable('activity', 'timestamp',
                                  if_not_exists => TRUE,
                                  migrate_data => TRUE);
        RAISE NOTICE '✓ Converted activity to hypertable';
    END IF;
END;
$$;

CREATE INDEX IF NOT EXISTS idx_activity_ts ON activity(timestamp);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    primary_app TEXT,
    frame_count INTEGER,
    total_ocr_words INTEGER,
    summary TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sessions_times ON sessions(start_time, end_time);

-- ===================================================================
-- STEP 6: Enable Compression (Optional)
-- ===================================================================

-- Enable compression on frames hypertable for data older than 48 hours
DO $$
BEGIN
    -- Set compression policy
    ALTER TABLE frames SET (
        timescaledb.compress,
        timescaledb.compress_segmentby = 'app_name'
    );

    -- Add compression policy (compress data older than 48 hours)
    PERFORM add_compression_policy('frames', INTERVAL '48 hours', if_not_exists => TRUE);

    RAISE NOTICE '✓ Enabled compression for frames (48 hour policy)';
EXCEPTION
    WHEN OTHERS THEN
        RAISE NOTICE '⚠ Could not enable compression: %', SQLERRM;
END;
$$;

-- ===================================================================
-- STEP 7: Verification
-- ===================================================================

DO $$
DECLARE
    ext_count INTEGER;
    hypertable_count INTEGER;
    index_count INTEGER;
BEGIN
    -- Check extensions
    SELECT COUNT(*) INTO ext_count FROM pg_extension
    WHERE extname IN ('timescaledb', 'vector');

    IF ext_count < 2 THEN
        RAISE EXCEPTION 'Missing extensions: expected 2, found %', ext_count;
    END IF;

    -- Check hypertables
    SELECT COUNT(*) INTO hypertable_count FROM timescaledb_information.hypertables
    WHERE hypertable_name IN ('frames', 'activity');

    IF hypertable_count < 2 THEN
        RAISE NOTICE '⚠ Expected 2 hypertables, found %', hypertable_count;
    END IF;

    -- Check indexes
    SELECT COUNT(*) INTO index_count FROM pg_indexes
    WHERE tablename = 'frames';

    RAISE NOTICE '';
    RAISE NOTICE '════════════════════════════════════════════════════════';
    RAISE NOTICE '✅ Schema Upgrade Complete';
    RAISE NOTICE '════════════════════════════════════════════════════════';
    RAISE NOTICE 'Extensions installed: %', ext_count;
    RAISE NOTICE 'Hypertables created: %', hypertable_count;
    RAISE NOTICE 'Indexes on frames: %', index_count;
    RAISE NOTICE '';
    RAISE NOTICE 'Next steps:';
    RAISE NOTICE '1. Test vector similarity search';
    RAISE NOTICE '2. Verify hypertable performance';
    RAISE NOTICE '3. Monitor compression progress';
    RAISE NOTICE '4. Restart capture pipeline';
END;
$$;

COMMIT;
