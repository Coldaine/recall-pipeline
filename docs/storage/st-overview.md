---
doc_type: architecture
subsystem: storage
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2026-02-09
---
# Storage Domain

## Decision: Postgres Only (Production)

**Why not SQLite?** Screenpipe's SQLite backend died from write contention. Multiple concurrent writes locked the database and it never recovered.

**Solution:** Single Postgres instance on a capable server. No local database complexity.

> **Note:** No SQLite. Postgres is the only storage engine. See `docs/architecture/overview.md` for rationale.

## Phase 1 Schema (Current Priority)

```sql
-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS vector;

-- Core capture table
CREATE TABLE frames (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    deployment_id   TEXT NOT NULL,
    captured_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Deduplication
    phash           BIGINT,
    phash_prefix    SMALLINT,  -- top 16 bits for fast filtering

    -- Image storage
    image_ref       TEXT,      -- path or URL to image

    -- Context (captured with frame)
    window_title    TEXT,
    app_name        TEXT,

    -- Lazy processing status (0=pending, 1=done, 2=error, 3=skipped)
    ocr_status      SMALLINT DEFAULT 0,
    ocr_text        TEXT,

    vision_status   SMALLINT DEFAULT 0,
    vision_summary  TEXT,

    embedding_status SMALLINT DEFAULT 0,
    embedding       vector(384)
);

-- Indexes
CREATE INDEX idx_frames_deployment ON frames (deployment_id);
CREATE INDEX idx_frames_captured_at ON frames (captured_at);
CREATE INDEX idx_frames_phash_prefix ON frames (phash_prefix);
CREATE INDEX idx_frames_ocr_pending ON frames (ocr_status) WHERE ocr_status = 0;
CREATE INDEX idx_frames_vision_pending ON frames (vision_status) WHERE vision_status = 0;
```

## Status Codes

| Code | Meaning |
|------|---------|
| 0 | Pending |
| 1 | Done |
| 2 | Error |
| 3 | Skipped |

## Phase 2+ Schema (Later)

These tables come after raw capture is working:

```sql
-- Phase 2: Search
CREATE INDEX idx_frames_ocr_fts ON frames
    USING GIN (to_tsvector('english', coalesce(ocr_text, '')));
CREATE INDEX idx_frames_embedding ON frames
    USING ivfflat (embedding vector_cosine_ops);

-- Phase 3: Hierarchical summaries
CREATE TABLE activities (...);
CREATE TABLE projects (...);
CREATE TABLE day_summaries (...);

-- Phase 4: Secrets
CREATE TABLE secrets (...);
```

## Image Storage

Options (decide based on what's easiest):
- **Filesystem:** Save images to disk, store path in `image_ref`
- **S3/R2:** Upload to object storage, store URL in `image_ref`
- **Postgres BYTEA:** Store directly in DB (simpler, but larger DB)

For Phase 1, filesystem is fine. Just get it working.

## Data Volume

- 12-18 hours capture/day
- ~2-5 GB/day raw
- Terabytes available — not a constraint
