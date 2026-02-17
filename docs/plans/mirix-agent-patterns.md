---
last_edited: 2026-02-17
editor: Amp (Claude Sonnet 4)
user: Coldaine
status: ready
version: 1.1.0
subsystem: architecture
tags: [mirix, agents, patterns, extraction, pre-deletion, memory, ocr, vision]
doc_type: reference
---

# MIRIX Agent Patterns — Extracted from `agents/` Before Deletion

> **Purpose**: Preserve architecturally valuable, Recall-specific patterns from the Python reference implementation (MemGPT/Letta fork) before the `agents/` directory is removed. These patterns inform the Pure Rust reimplementation (ADR-009).
>
> **Scope**: Only patterns specific to screen capture recall. Generic MemGPT/Letta framework code (ORM, SDK, server, prompts, tool infrastructure) is excluded.
>
> **⚠ Reference Code Bugs**: The Python reference contains implementation bugs (noted inline with ⚠). The Rust reimplementation should follow the *intended* design, not reproduce the bugs.

---

## Northstar Compliance Checklist

Before reimplementing any pattern, verify against [Northstar.md](../Northstar.md):

- [ ] `deployment_id` on every table
- [ ] Status codes: `0=Pending, 1=Done, 2=Error, 3=Skipped`
- [ ] Lazy processing only (never block capture)
- [ ] Hierarchy: Frame → Activity → Project → Day Summary
- [ ] PostgreSQL only (no SQLite)
- [ ] Secrets encrypted with FIDO2 key

---

## 1. Data Models

### 1.1 Frame Schema (Authoritative: SQL)

**Source**: `agents/database/upgrade_schema_extensions.sql` (authoritative), `agents/schemas/frame.py` (Pydantic subset)

The Frame is the atomic unit of capture — one screenshot at a point in time. The authoritative schema from the SQL migration is:

```sql
frames (
    id                UUID NOT NULL,
    captured_at       TIMESTAMPTZ NOT NULL,     -- When the screenshot was taken
    window_title      TEXT,                      -- Active window title
    app_name          TEXT,                      -- Foreground application name
    image_ref         TEXT,                      -- Path or file:// URI to image on disk
    phash             BIGINT,                    -- 64-bit perceptual hash for dedup
    phash_prefix      SMALLINT,                  -- First 16 bits of phash (fast candidate filter)
    ocr_text          TEXT,                      -- Extracted text (populated by OCR worker)
    ocr_status        SMALLINT DEFAULT 0,        -- Northstar: 0=Pending, 1=Done, 2=Error, 3=Skipped
    embedding         VECTOR(384),               -- pgvector embedding
    embedding_status  SMALLINT DEFAULT 0,        -- Northstar status
    vision_summary    TEXT,                       -- LLM-generated summary
    vision_status     SMALLINT DEFAULT 0,         -- Northstar status
    created_at        TIMESTAMPTZ DEFAULT now(),
    deployment_id     TEXT,                       -- Which machine captured this
    image_size_bytes  INTEGER,
    has_text          BOOLEAN DEFAULT FALSE,      -- Set by OCR worker
    has_activity      BOOLEAN DEFAULT FALSE,
    activity_id       UUID,                       -- FK to activities (session grouping)
    PRIMARY KEY (id, captured_at)                 -- Composite PK for TimescaleDB hypertable
)
```

**Key design decisions**:
- `phash` + `phash_prefix` enables two-tier dedup: fast `SMALLINT` filter on `phash_prefix`, then exact Hamming distance on `phash`
- Three separate status columns (`ocr_status`, `vision_status`, `embedding_status`) — one per processing stage
- `has_text` is a denormalized boolean set by the OCR worker for fast filtering
- `image_ref` stores a path/URI, not the image blob
- `activity_id` links frames to their parent activity/session
- Composite PK `(id, captured_at)` is required for TimescaleDB hypertable partitioning
- TimescaleDB compression enabled with 48-hour policy, segmented by `app_name`

**⚠ Reference inconsistency**: The Pydantic model in `schemas/frame.py` defines different status semantics (`0=pending, 1=processed, 2=failed`) that don't match the SQL schema or Northstar. The Rust implementation MUST use the Northstar status codes consistently.

**Reimplementation target**: Rust (this is the core capture schema, already partially in `capture/`)

### 1.2 Deduplication Contract

**Source**: `capture/recall-store/src/postgres.rs`, `capture/recall-capture/src/dedup.rs`

Two-tier dedup, both already in Rust:

| Tier | Method | Threshold | Scope |
|------|--------|-----------|-------|
| In-memory | Histogram + SSIM vs previous frame | `< 0.006` (lower = more similar) | Adjacent frames only |
| Database | pHash Hamming distance within time window | `≤ 10` bits | Last N seconds (default: 10s, configurable `--dedup-window-secs`) |

The database check filters candidates by `phash_prefix` (SMALLINT, first 16 bits) for fast narrowing before computing full Hamming distance.

**Reimplementation target**: Already in Rust — preserve as-is.

### 1.3 OCR Result Schema

**Source**: `agents/processors/ocr_worker.py`, `agents/database/upgrade_schema_extensions.sql`

```sql
ocr_text (
    id          BIGSERIAL PRIMARY KEY,
    frame_id    UUID NOT NULL REFERENCES frames(id) ON DELETE CASCADE,
    text        TEXT,
    confidence  REAL,          -- Average Tesseract word-level confidence (0–100)
    language    TEXT,           -- e.g., "eng"
    bbox        JSONB,         -- Bounding box data (optional)
    created_at  TIMESTAMPTZ DEFAULT now()
)
```

OCR results are written to two places:
1. `frames.ocr_text` and `frames.has_text` — denormalized for fast queries
2. `ocr_text` table — detailed records with confidence and bounding boxes

**Confidence computation**: Average of all word-level Tesseract confidences where `conf > 0`. Words with `conf <= 0` are excluded.

**Reimplementation target**: Rust

### 1.4 Related Tables

**Source**: `agents/database/upgrade_schema_extensions.sql`

```sql
window_context (
    id            BIGSERIAL PRIMARY KEY,
    frame_id      UUID NOT NULL REFERENCES frames(id) ON DELETE CASCADE,
    app_name      TEXT,
    window_title  TEXT,
    process_name  TEXT,
    is_focused    BOOLEAN,
    url           TEXT,            -- Browser URL if applicable
    created_at    TIMESTAMPTZ DEFAULT now()
)

activity (                         -- TimescaleDB hypertable
    id              BIGSERIAL PRIMARY KEY,
    timestamp       TIMESTAMPTZ NOT NULL,
    activity_type   TEXT,
    keystroke_count INTEGER,
    mouse_events    INTEGER,
    idle_seconds    INTEGER
)

sessions (
    id          UUID PRIMARY KEY,
    start_time  TIMESTAMPTZ NOT NULL,
    end_time    TIMESTAMPTZ NOT NULL,
    primary_app TEXT,
    frame_count INTEGER,
    total_ocr_words INTEGER,
    summary     TEXT,
    created_at  TIMESTAMPTZ DEFAULT now()
)
```

All child tables of `frames` use `ON DELETE CASCADE` — when retention policy deletes old frames, metadata is automatically purged.

**Reimplementation target**: Rust (Postgres migrations)

### 1.5 MIRIX Memory Type Schemas

All memory types share a common pattern with `tree_path` categorization and embedding support. These are for the *agentic memory layer* that will eventually consume capture data.

| Type | ID Prefix | Key Fields | Purpose |
|------|-----------|------------|---------|
| **Episodic** | `ep_mem` | `event_type`, `summary`, `details`, `actor`, `occurred_at`, `tree_path` | Event timeline — "what happened and when" |
| **Semantic** | `sem_item` | `name`, `summary`, `details`, `source`, `tree_path` | Extracted facts and knowledge |
| **Procedural** | `proc_item` | `entry_type`, `summary`, `steps: List[str]`, `tree_path` | Workflows and how-tos |
| **Resource** | `res_item` | `title`, `summary`, `resource_type`, `content`, `tree_path` | Files, links, docs |
| **Knowledge Vault** | `kv_item` | `entry_type`, `source`, `sensitivity`, `secret_value`, `caption` | Credentials, sensitive data |

**Cross-cutting patterns**:
- `tree_path: List[str]` — hierarchical categorization (e.g., `['work', 'projects', 'ai-research']`). Maps to Postgres `text[]` with GIN indexing.
- All embeddings are zero-padded to `MAX_EMBEDDING_DIM = 4096` to guarantee pgvector column uniformity regardless of source model.
- `last_modify: Dict` — tracks `{timestamp, operation}` for audit trail.

**⚠ Knowledge Vault & Northstar**: Northstar requires secrets to be "stored encrypted (requiring FIDO2 key to decrypt)". The Python reference stores `secret_value` as plaintext with only a `sensitivity` label. The Rust reimplementation MUST encrypt `secret_value` at rest with FIDO2-gated decryption.

**Reimplementation target**: Rust structs + pgvector (medium priority)

---

## 2. Processing Patterns

### 2.1 Status Code Convention (Northstar Canonical)

**Source**: `Northstar.md` §5

All processing status columns MUST use this enum:

| Value | Meaning |
|-------|---------|
| `0` | **Pending** — not yet processed |
| `1` | **Done** — successfully processed |
| `2` | **Error** — processing failed |
| `3` | **Skipped** — intentionally not processed |

**⚠ Python reference divergence**: The OCR and Vision workers in `agents/` use different status progressions (`0→1→2→3→4` and `-1` for error). The `or-overview.md` documentation correctly specifies Northstar codes. The Rust reimplementation MUST follow Northstar exclusively.

**"In-progress" representation**: Northstar doesn't define an "in progress" state. For the Rust implementation, use a **lease-based claim** pattern instead of a status value:
- Add optional `claimed_at TIMESTAMPTZ` and `claimed_by TEXT` columns per stage
- A frame with `status=0` and `claimed_at IS NOT NULL` and `claimed_at > now() - interval 'N minutes'` is "in progress"
- Stale claims (older than N minutes) are automatically re-eligible

### 2.2 Worker Queue Pattern (Poll-Loop + Atomic Claim)

**Source**: `agents/processors/ocr_worker.py`, `agents/processors/vision_worker.py`

All workers follow the same architectural pattern:

```
┌──────────────────────────────────────────────────────────┐
│                   Async Poll-Loop Worker                  │
│                                                           │
│  1. ATOMIC CLAIM (single statement, single transaction):  │
│     UPDATE frames                                         │
│     SET claimed_at = now(), claimed_by = $worker_id       │
│     WHERE id IN (                                         │
│       SELECT id FROM frames                               │
│       WHERE ocr_status = 0                                │
│         AND (claimed_at IS NULL                           │
│          OR claimed_at < now() - interval 'N min')        │
│       ORDER BY captured_at ASC                            │
│       LIMIT $batch_size                                   │
│       FOR UPDATE SKIP LOCKED                              │
│     )                                                     │
│     RETURNING *                                           │
│                                                           │
│  2. Process each frame (OCR / Vision / Embedding)         │
│                                                           │
│  3. In a transaction, write results:                      │
│     - Update status → 1 (Done) or 2 (Error)              │
│     - Clear claimed_at, claimed_by                        │
│     - Write output data                                   │
│                                                           │
│  4. If nothing claimed → sleep(poll_interval)             │
│     Else → immediately poll again                         │
└──────────────────────────────────────────────────────────┘
```

**⚠ Python reference bug**: The Python workers use separate `SELECT FOR UPDATE SKIP LOCKED` and `UPDATE SET status=processing` statements without wrapping them in an explicit transaction. In `asyncpg`, each statement runs in its own implicit transaction, so the row lock is released before the status update — creating a race condition where multiple workers can claim the same frame. The Rust implementation MUST use the atomic single-statement pattern above (or an explicit transaction).

**Key configuration defaults**:

| Parameter | Default | Notes |
|-----------|---------|-------|
| `batch_size` | 10 | Frames per polling cycle |
| `poll_interval` | 5.0s | Sleep between empty polls |
| `max_retries` | 3 | For transient DB errors |
| `retry_delay` | 1.0s | Base delay, exponential backoff (`delay × 2^attempt`) |

**Reimplementation target**: Rust. This is the core lazy processing pattern — critical to get right.

### 2.3 OCR Worker Specifics

**Source**: `agents/processors/ocr_worker.py`

- Uses Tesseract via `pytesseract`
- Configurable language: `tesseract_lang` (default: `"eng"`, supports multi-lang e.g. `"eng+spa"`)
- Extracts word-level data via `image_to_data()` with `Output.DICT`
- Confidence: average of all word-level confidences where `conf > 0`
- `min_text_length = 1` — minimum chars to set `has_text = true`
- Writes to both `frames.ocr_text` (denormalized) and `ocr_text` table (detailed, with `ON CONFLICT DO NOTHING`)

**Reimplementation target**: Rust. Use `leptess` or `tesseract-rs` crate.

### 2.4 Vision Worker Specifics (LLM Summarization)

**Source**: `agents/processors/vision_worker.py`

Processes OCR-done frames to generate 1-2 sentence activity summaries.

- Loads image, encodes as base64 data URI
- Sends to LLM Vision API with OCR context (truncated to 1000 chars)
- Rate-limited: `rate_limit_delay = 0.5s` between API calls

**Vision prompt template**:
```
You are analyzing a screenshot from a user's computer.
The OCR extracted text is: {ocr_text}

Describe concisely (1-2 sentences) what application/window is visible
and what the user is likely doing. Focus on the activity, not UI elements.
```

| Parameter | Default | Notes |
|-----------|---------|-------|
| `model` | `"gpt-4o"` | LLM model for vision |
| `max_tokens` | 150 | Keep summaries short |
| `rate_limit_delay` | 0.5s | Between API calls |

**Reimplementation target**: Rust. Use `reqwest` for LLM API calls.

### 2.5 Embedding Worker (Planned)

**Source**: `agents/schemas/frame.py` (`embedding_status` field), `docs/orchestration/or-overview.md`

No standalone embedding worker exists in the Python reference. The schema has `embedding VECTOR(384)` and `embedding_status SMALLINT DEFAULT 0`, and `or-overview.md` documents it as a planned worker:

> | Embedding Worker | Frames where `embedding_status = 0` | `embedding` vector, update `embedding_status = 1` |

Embeddings are currently only generated within memory agent managers (episodic, semantic, etc.), not for raw frames.

**Reimplementation target**: Rust. Should follow the same poll-loop pattern as OCR/Vision workers.

---

## 3. Memory Architecture

### 3.1 Three-Tier Memory Model

**Source**: `agents/memory.py`, `agents/schemas/memory.py`

The system uses three tiers of memory (from MemGPT design, applied to screen capture):

| Tier | Description | Screen Capture Mapping |
|------|-------------|----------------------|
| **Core Memory** | Always in-context, editable by agent. Limited to 5000 chars per block. Contains `persona` and `human` blocks. | System identity, user preferences, persistent configuration |
| **Recall Memory** | Searchable conversation/event history. | Frame OCR text, vision summaries, conversation logs |
| **Archival Memory** | Long-term vector store, searchable via embeddings. | Hierarchical summaries, extracted knowledge, semantic facts |

### 3.2 Message Summarization

**Source**: `agents/memory.py`

When conversation context approaches 75% of the context window, messages are summarized recursively:

| Parameter | Value | Notes |
|-----------|-------|-------|
| `memory_warning_threshold` | 0.75 | Start summarizing at 75% of context window |
| `desired_memory_token_pressure` | 0.1 | Target 10% usage after summarization |
| `keep_last_n_messages` | 5 | Preserved as in-context examples |
| `max_summarizer_retries` | 3 | Fatal error if can't compress |

If the summary itself exceeds the threshold, it splits the message sequence and recurses on the first portion, then concatenates.

**Reimplementation target**: Rust (low priority — needs LLM client)

### 3.3 Multi-Agent Routing Concept

**Source**: `agents/agent/*.py`, `agents/constants.py`

The Meta Memory Agent orchestrates specialist agents: it receives update signals and routes to the appropriate specialist (Episodic, Semantic, Procedural, Resource, Knowledge Vault, Core). Each specialist writes to its own memory store.

The key concept worth preserving is the **routing pattern** — a coordinator dispatches work to type-specific handlers. In Rust, this maps to a trait-based system where each handler implements a `MemoryAgent` trait with `insert`, `update`, `search` methods.

**Reimplementation target**: Rust (low priority — trait-based design)

---

## 4. Hierarchical Summarization

### 4.1 Schema

**Source**: `agents/database/create_hierarchy_tables.sql`

```
Frame → Activity → Project → Day Summary
```

```sql
activities (
    id              VARCHAR PRIMARY KEY,
    deployment_id   VARCHAR NOT NULL,
    start_at        TIMESTAMPTZ,
    end_at          TIMESTAMPTZ,
    summary         TEXT,
    project_id      VARCHAR              -- FK to projects
)
-- Indexed: deployment_id, project_id, (start_at, end_at)

projects (
    id              VARCHAR PRIMARY KEY,
    name            VARCHAR,
    summary         TEXT
)
-- Indexed: name

day_summaries (
    id              VARCHAR PRIMARY KEY,
    deployment_id   VARCHAR,             -- NULL for cross-deployment summaries
    date            DATE NOT NULL,
    summary         TEXT,
    UNIQUE(deployment_id, date)
)
-- Indexed: deployment_id, date
```

**⚠ Northstar compliance issue**: `projects` table has no `deployment_id`. Northstar says "deployment_id on every table". Decision needed:
- **Option A**: Add `deployment_id` to `projects` (simple, compliant). Projects that span machines get a NULL or sentinel value.
- **Option B**: Amend Northstar to allow a small set of global/cross-deployment tables. Requires explicit exception documentation.

### 4.2 Rollup Pattern

**Source**: `agents/processors/consolidator.py` (stub), schema

1. **Frame → Activity**: Group consecutive frames by `app_name`/`window_title` with a **5-minute gap threshold** (`gap_threshold_secs = 300`). Summarize each activity window using LLM.
2. **Activity → Project**: Cluster activities into projects by detected project name/context. Summarize project progress.
3. **Activity/Project → Day Summary**: Roll up all activities for a `(deployment_id, date)` pair into a day summary. Cross-deployment summaries use NULL `deployment_id`.

The consolidator is a stub in the Python reference but the schema and gap-detection logic are sound.

**Reimplementation target**: Rust (high priority — core feature)

---

## 5. Image Lifecycle & Retention

**Source**: `capture/src/bin/recall.rs`, `capture/recall-store/src/images.rs`

Already implemented in Rust:

| Policy | Detail |
|--------|--------|
| **Retention period** | 30 days (configurable) |
| **DB cleanup** | `DELETE FROM frames WHERE captured_at < $cutoff` |
| **Disk cleanup** | Iterates date-based directories (`YYYY-MM-DD`), deletes folders older than cutoff |
| **Cascade** | `ON DELETE CASCADE` on `ocr_text`, `window_context` ensures metadata cleanup |
| **Schedule** | Daily background task (24h interval) in capture daemon |
| **Compression** | TimescaleDB compression policy: 48 hours, segmented by `app_name` |

**Reimplementation target**: Already in Rust — preserve as-is.

---

## 6. Constants Worth Preserving

### 6.1 Embedding Constants

| Constant | Value | Notes |
|----------|-------|-------|
| `MAX_EMBEDDING_DIM` | 4096 | Memory schema padding target — **DO NOT CHANGE** |
| Frame embedding dimension | 384 | `VECTOR(384)` in frames table |
| `DEFAULT_EMBEDDING_CHUNK_SIZE` | 300 | Tokens per chunk for text splitting |

### 6.2 Processing Constants

| Constant | Value | Notes |
|----------|-------|-------|
| `MAX_IMAGES_TO_PROCESS` | 100 | Batch ceiling for image processing |
| Dedup threshold (pHash) | ≤ 10 bits | Hamming distance |
| Dedup threshold (visual) | < 0.006 | Histogram + SSIM combined |
| Dedup time window | 10s | Configurable via `--dedup-window-secs` |
| Session gap threshold | 300s | 5 minutes between frames = new session |

### 6.3 Memory Limits

| Constant | Value | Notes |
|----------|-------|-------|
| `CORE_MEMORY_BLOCK_CHAR_LIMIT` | 5000 | Per memory block |
| `memory_warning_threshold` | 0.75 | Start summarizing at 75% context |
| `desired_memory_token_pressure` | 0.1 | Target 10% after summarization |
| `keep_last_n_messages` | 5 | Preserved after summarization |

### 6.4 Status Codes (Northstar Canonical)

```
0 = Pending
1 = Done
2 = Error
3 = Skipped
```

---

## 7. Reimplementation Summary

| Pattern | Target | Priority | Notes |
|---------|--------|----------|-------|
| Frame schema + extensions SQL | Rust migration | ✅ Exists | Verify status columns match Northstar |
| Dedup (pHash + visual) | Rust | ✅ Exists | Already in `capture/` |
| Image retention (30d + cascade) | Rust | ✅ Exists | Already in `capture/` |
| OCR poll-loop worker | Rust | **High** | Fix atomic claim pattern |
| Vision poll-loop worker | Rust | **High** | LLM API client needed |
| Embedding poll-loop worker | Rust | **High** | Schema exists, no Python impl |
| Hierarchy tables (SQL) | Rust migration | **High** | Fix `deployment_id` on `projects` |
| Session gap detection (300s) | Rust | **Medium** | Simple time-gap logic |
| Memory type schemas (5 types) | Rust + pgvector | **Medium** | `tree_path` → `text[]` + GIN |
| Embedding padding to 4096 | Rust | **Medium** | Shared utility |
| Knowledge Vault encryption | Rust | **Medium** | FIDO2-gated, per Northstar |
| Day summary rollup | Rust | **Medium** | LLM summarization |
| Message summarization | Rust | Low | Needs LLM client |
| Multi-agent routing | Rust | Low | Trait-based coordinator pattern |
