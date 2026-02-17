---
doc_type: plan
subsystem: general
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# Worklog

- Log daily/weekly progress snapshots: what changed, blockers, next steps.
- Required for agentic updates: every PR touching agents/routing/storage must add a brief entry.

---

## 2026-02-09: Screenpipe Removal & Clean Architecture

### What Changed
- Removed all 10 screenpipe-* crates from capture/
- Created 3 clean crates: recall-capture, recall-db, recall-store
- recall-capture uses xcap 0.8 directly (full Wayland/PipeWire support)
- Preserved all custom Postgres schema, Storage trait, dedup logic, image storage
- Removed legacy scripts and generated docs from repo root
- Binary renamed from recall-rs to recall

### Key Decisions
- **Option D**: Build minimal capture from scratch rather than maintaining screenpipe fork
- **xcap 0.8**: Direct dependency instead of screenpipe wrapper (Wayland + PipeWire support via XDG Desktop Portal)
- **No screenpipe dependency**: The useful code was ~500 lines; the baggage was 10 crates of unwanted functionality
- **No local database**: Frames POST directly to Postgres on server, matching original architecture vision

### Next Steps
1. Get `cargo check --workspace` passing
2. Wire up HTTP POST for remote frame submission
3. Build server-side ingestion endpoint
4. Implement hierarchical summarization pipeline

---

## 2025-11-28: CI Setup & Screenpipe Crate Audit

### What Changed
- Registered new self-hosted GitHub Actions runner at `/home/coldaine/actions-runner-recall/`
- Created CI workflow (`.github/workflows/ci.yml`) with Python lint/test and Rust check jobs
- PR #51 opened for CI integration
- Full audit of all 10 screenpipe crates (see `docs/architecture/screenpipe-crate-audit.md`)

### Key Decisions
- **Delete screenpipe-audio**: Broken (18+ errors), not needed for project goals
- **Delete screenpipe-server**: 50+ endpoints overkill for single-user, heavy audio coupling
- **Keep screenpipe-vision**: Core capture functionality
- **Keep screenpipe-storage**: Deduplication and frame management essential
- **Windows + Linux only**: macOS code can be stripped
- **Frame-based capture**: Simpler than video-based approach

### Blockers
- Rust build fails due to: rocksdb/clang 21 incompatibility, screenpipe-db type errors, zbus compilation
- screenpipe-audio must be removed before other cleanup can proceed

### Next Steps
1. Execute screenpipe-audio removal plan (documented in audit)
2. Evaluate screenpipe-server removal after audio cleanup
3. Fix screenpipe-db compilation errors
4. Get `cargo check --workspace` passing
