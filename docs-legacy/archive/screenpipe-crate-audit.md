# Screenpipe Crate Audit & Removal Plan

**Date**: 2025-11-28
**Status**: In Progress
**Related PR**: #51 (CI Setup)

## Background

This project forked [screenpipe](https://github.com/mediar-ai/screenpipe) because:

1. **Upstream direction issues**: Screenpipe is moving toward paid apps and features not aligned with this project's goals
2. **Database issues**: Their database layer doesn't work well for our use case
3. **Scope mismatch**: We need a simpler, single-user capture system, not a full-featured product

The fork was intended to extract useful capture/dedup/OCR functionality, but the codebase accumulated breakage and hasn't been actively maintained since forking.

## Current Build Status

As of 2025-11-28, `cargo check --workspace` fails with:

### Critical Errors

| Crate | Error | Root Cause |
|-------|-------|------------|
| `screenpipe-db` | `sqlite3_vec_init` transmute type mismatch | Incorrect transmute type specification |
| `screenpipe-db` | `MigrateError` vs `sqlx::Error` | Missing error type conversion |
| `zbus` | Compilation error | Upstream dependency issue (Linux D-Bus bindings) |
| `rocksdb` | clang 21 incompatibility | Native dependency issue with system clang |
| `screenpipe-audio` | 18+ errors | Duplicate modules, broken code, not maintained |

### Partial Fixes Applied

1. **Benchmark files**: Commented out missing `[[bench]]` sections in multiple Cargo.toml files
2. **core-foundation version**: Relaxed `=0.10.0` pin to `0.10` to resolve dependency conflicts

## Crate Inventory

Full audit of all screenpipe crates with line counts and recommendations:

### screenpipe-audio (~3000 lines) - **DELETE**

**Status**: Broken, 18+ compilation errors
**Purpose**: Audio capture, transcription via whisper-rs

**Issues**:
- Duplicate module definitions (`audio_transcription_engine`, `multilingual`, etc.)
- Type mismatches throughout
- No longer needed for project goals

**Recommendation**: Remove entirely. See [Removal Plan](#screenpipe-audio-removal-plan) below.

---

### screenpipe-server (~4000 lines) - **DELETE**

**Status**: Compiles but vastly overbuilt
**Purpose**: HTTP REST API server

**Components**:
- `server.rs`: 50+ REST endpoints (overkill for single-user)
- `cli.rs`: CLI argument parsing with audio options
- Health checks, video streaming, timeline APIs, plugin system

**Why Delete**:
- Single-user project doesn't need 50+ HTTP endpoints
- Video streaming approach is overkill (frame-based approach is simpler)
- Heavy coupling to audio crate
- Python agents can handle orchestration better

**Recommendation**: Delete after removing audio crate. No salvageable functionality for our architecture.

---

### screenpipe-events (~450 lines) - **EVALUATE (Partial Keep)**

**Status**: Compiles
**Purpose**: Event pub/sub system + meeting detection

**Components**:
- `events.rs` (~305 lines): Generic pub/sub pattern - **KEEP**
- `meetings.rs` (~95 lines): Detects Google Meet, Zoom, Teams via audio - **DELETE**

**Why Partial**:
- The pub/sub pattern (`EventEmitter<T>`) is useful for decoupling
- Meeting detection depends on audio crate (which we're deleting)

**Recommendation**: Extract pub/sub pattern, delete meeting detection.

---

### screenpipe-vision (~2000 lines) - **KEEP**

**Status**: Compiles with warnings
**Purpose**: Screen capture, OCR, window context

**Components**:
- `core.rs`: Main capture logic
- `capture_*`: Platform-specific capture (Linux, Windows, macOS)
- `utils.rs`: Helper functions
- `browser_data.rs`: Browser URL extraction
- `run_ui.rs`: UI automation for accessibility

**Dependencies**:
- `xcap`: Cross-platform screen capture
- `rusty-tesseract`: OCR
- Platform-specific: `atspi` (Linux), `uiautomation` (Windows), `cidre` (macOS)

**Recommendation**: Keep. This is core capture functionality we need.

**Notes**:
- macOS code can be removed (Windows + Linux only)
- Benchmarks are commented out (missing source files)

---

### screenpipe-storage (~1800 lines) - **KEEP**

**Status**: Compiles
**Purpose**: Frame caching, deduplication, image handling

**Key Features**:
- Perceptual hashing (phash) for frame deduplication
- Frame cache with configurable sizes
- Image compression/conversion
- Timestamp tracking

**Recommendation**: Keep. Deduplication and frame management are essential.

---

### screenpipe-db (~3567 lines) - **EVALUATE**

**Status**: 2 compilation errors
**Purpose**: Database abstraction (SQLite + Postgres)

**Components**:
- `db.rs` (~2800 lines): SQLite operations, sqlite3_vec integration
- `postgres.rs` (~200 lines): Postgres operations
- `migration_worker.rs`: Async migration handling
- `types.rs`: Shared types

**Issues**:
1. `sqlite3_vec_init` transmute needs fix:
   ```rust
   // Current (broken):
   std::mem::transmute::<*const (), unsafe extern "C" fn()>(sqlite3_vec_init as *const ())

   // Fix: Remove explicit type, let compiler infer
   std::mem::transmute(sqlite3_vec_init as *const ())
   ```

2. Postgres migration error type needs wrapping:
   ```rust
   // Add .map_err() wrapper
   .map_err(|e| sqlx::Error::Migrate(Box::new(e)))
   ```

**Recommendation**: Fix compilation errors, then evaluate if we need both SQLite and Postgres paths or can simplify to Postgres-only.

---

### screenpipe-core (~3854 lines) - **EVALUATE**

**Status**: Compiles
**Purpose**: Shared utilities, FFmpeg, language detection, LLM integration

**Components**:
- `ffmpeg.rs` (~800 lines): Video encoding/decoding
- `llm.rs` (~270 lines): LLM integration utilities
- `pipes.rs` (~700 lines): Plugin/pipe system
- `language.rs` (~130 lines): Language detection
- `google_ai.rs`, `deno.rs`, etc.

**Why Evaluate**:
- FFmpeg module is complex (video-based approach)
- LLM integration might conflict with Python agents
- Pipes system might be overkill

**Recommendation**: Audit each module individually. Keep language detection and basic utilities, potentially remove FFmpeg and complex LLM/pipe systems.

---

### screenpipe-integrations (~188 lines) - **EVALUATE**

**Status**: Compiles
**Purpose**: Cloud OCR integrations

**Components**:
- `unstructured.rs`: Unstructured.io cloud OCR API

**Recommendation**: Keep if cloud OCR fallback is desired. Low complexity.

---

### screenpipe-llm (~833 lines) - **EVALUATE**

**Status**: Compiles
**Purpose**: LLM budget management, embeddings, vision

**Components**:
- `budget.rs`: Token budget tracking
- `embeddings.rs`: Vector embeddings (uses `fastembed-rs`)
- `vision.rs`: Vision LLM integration

**Recommendation**: Evaluate overlap with Python agents. May be redundant if Python handles LLM coordination.

---

### screenpipe-memory (~160 lines) - **EVALUATE**

**Status**: Compiles
**Purpose**: Memory stub (minimal implementation)

**Recommendation**: Either expand or remove. Currently too minimal to be useful.

---

## screenpipe-audio Removal Plan

### Touchpoints Identified

1. **Workspace Cargo.toml** (`capture/Cargo.toml`)
   - Remove from `members` list
   - Remove from `[dependencies]` section if present

2. **screenpipe-server/Cargo.toml**
   - Remove `screenpipe-audio` dependency

3. **screenpipe-server/src/server.rs** (~100+ audio references)
   - Remove audio imports
   - Remove audio-related handler functions
   - Remove audio routes from router
   - Remove audio structs/types

4. **screenpipe-server/src/cli.rs**
   - Remove audio CLI options (e.g., `--audio-device`, `--audio-disabled`)
   - Remove audio-related argument parsing

5. **screenpipe-server/src/bin/screenpipe-server.rs**
   - Remove audio initialization code
   - Remove audio device handling

6. **screenpipe-server/tests/**
   - Remove or update tests that reference audio

7. **screenpipe-audio directory**
   - Delete entire `capture/screenpipe-audio/` directory

### Removal Order

Execute in this order to maintain compilability at each step:

1. Remove audio from workspace Cargo.toml
2. Remove audio dependency from screenpipe-server/Cargo.toml
3. Clean server.rs audio imports and code
4. Clean cli.rs audio imports
5. Clean main binary audio imports
6. Clean/remove audio tests
7. Delete screenpipe-audio directory
8. Run `cargo check --workspace` to verify

---

## Architecture Decisions

### Frame-Based vs Video-Based Capture

**Decision**: Frame-based approach

**Rationale**:
- Simpler implementation
- Better deduplication via phash (compare frames directly)
- No need for FFmpeg encoding/decoding complexity
- Sufficient for recall use case (we care about content, not motion)

**Trade-off**: Lose smooth playback, but that's not a requirement.

### Platforms Supported

**Decision**: Windows + Linux only

**Rationale**:
- macOS code can be stripped (no macOS machines to support)
- Simplifies dependency tree
- Reduces platform-specific debugging surface

### OCR Strategy

**Decision**: Lazy OCR by default

**Current approach** (from architecture):
- Laptop: Lazy (upload raw frames)
- Desktop: Real-time (has spare CPU)
- Server: Batched processing

**Tools**: Tesseract (local), Unstructured.io (cloud fallback)

---

## CI Status

### Self-Hosted Runner

- **Location**: `/home/coldaine/actions-runner-recall/`
- **Status**: Running as systemd service (`actions.runner.Coldaine-recall-pipeline.laptop-extra.service`)
- **Labels**: `self-hosted, Linux, X64, recall-pipeline`

### Workflow

PR #51 created `.github/workflows/ci.yml`:

```yaml
jobs:
  python:
    # Lint + test with uv
    runs-on: [self-hosted, Linux, X64, recall-pipeline]

  rust:
    # cargo check (continue-on-error due to native deps)
    runs-on: [self-hosted, Linux, X64, recall-pipeline]
```

Rust job is informational only until crate cleanup is complete.

---

## How to Resume

### Immediate Next Steps

1. **Complete screenpipe-audio removal**
   - Follow the [Removal Plan](#screenpipe-audio-removal-plan) above
   - After removal, screenpipe-server may have cascade errors

2. **Evaluate screenpipe-server removal**
   - If audio removal leaves server too broken, delete entire crate
   - Document any useful patterns before deletion

3. **Fix screenpipe-db compilation**
   - Apply the two fixes documented above
   - Verify `cargo check` passes for this crate

4. **Evaluate remaining crates**
   - Review screenpipe-core modules individually
   - Remove FFmpeg if video approach abandoned
   - Remove LLM modules if Python agents handle this

### Medium-Term Goals

1. Get `cargo check --workspace` passing
2. Create minimal capture daemon (vision + storage + db)
3. Integrate with Python agents for orchestration

### Decision Points

Before resuming, confirm:

1. **Do we need both SQLite and Postgres?** Current architecture shows local SQLite cache + central Postgres. Is local cache necessary?

2. **What handles LLM coordination?** Rust (screenpipe-llm) or Python (agents/)? Pick one.

3. **Do we need the pipes/plugin system?** screenpipe-core has a complex pipe system. Worth keeping or simplify?

---

## Reference: Crate Line Counts

| Crate | Lines | Verdict |
|-------|-------|---------|
| screenpipe-audio | ~3000 | DELETE |
| screenpipe-server | ~4000 | DELETE |
| screenpipe-events | ~450 | PARTIAL (keep pub/sub) |
| screenpipe-vision | ~2000 | KEEP |
| screenpipe-storage | ~1800 | KEEP |
| screenpipe-db | ~3567 | FIX & EVALUATE |
| screenpipe-core | ~3854 | EVALUATE (per module) |
| screenpipe-integrations | ~188 | KEEP (low cost) |
| screenpipe-llm | ~833 | EVALUATE |
| screenpipe-memory | ~160 | EVALUATE |

**Total**: ~19,850 lines
**After removal (audio + server)**: ~12,850 lines
**Dependencies**: 987 (to be reduced after cleanup)
