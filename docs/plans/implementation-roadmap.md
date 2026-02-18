---
doc_type: plan
subsystem: orchestration
version: 1.0.0
status: active
owners: Coldaine
last_reviewed: 2026-02-17
---

# Implementation Roadmap: Recall Pipeline (Rust Port)

> **Steering Document**: This plan governs the migration from the Python prototype to the production-grade Pure Rust architecture (ADR-009).

## 🎯 Executive Goals
1.  **Reliability**: Fix critical data integrity issues (timestamps) immediately.
2.  **Performance**: Establish and enforce strict resource budgets.
3.  **Parity**: Systematically port MIRIX patterns (`docs/plans/mirix-agent-patterns.md`) to Rust workers.
4.  **Autonomy**: Enable "Local CI" workflows to bypass cloud dependency blockers.

---

## 🚦 Performance Budgets & Limits

These are **hard constraints** for the Rust implementation.

| Metric | Budget | Rationale |
| :--- | :--- | :--- |
| **Capture Latency** | < 16ms (Capture) / < 500ms (DB Commit) | Capture must be real-time (60fps capable); persistence is async but near-time. |
| **CPU Overhead** | < 5% of 1 Core (Idle/Capture) | Background process must not slow down the user's work. |
| **RAM Footprint** | < 200MB (RSS) | Lean runtime; avoid Python's heavy memory baseline. |
| **Storage Growth** | ~1GB / day (Active use) | Assumes deduplication and compressed image storage. |
| **Cold Start** | < 100ms | System must be instantly available on wake. |

## 🛠️ Tooling & Extensions (Requirements)

The following Gemini CLI extensions and tools are required for development and orchestration:

- **Conductor**: Proactive project management and context tracking.
- **Stitch**: Official UI/UX design and code generation via MCP.
- **MCP Toolbox for Databases**: Structured context and tool generation for PostgreSQL.
- **mcp-cli**: Dynamic discovery and interaction with MCP servers (Pattern: CLI-based tool interaction).

---

## 🛠️ PR Testing Phases (Local CI)

Due to current cloud CI limitations, all PRs must pass **Local CI** gates.

**Protocol:**
1.  **Tier 1 (Fast - Pre-Commit):** `cargo fmt --check && cargo clippy`
2.  **Tier 2 (Logic - Pre-Push):** `cargo test --lib` (Unit tests)
3.  **Tier 3 (Integration - Pre-Merge):** `just test-integration` (Requires local Postgres)

**Definition of Done (DoD) for Tasks:**
*   [ ] Code implements the spec.
*   [ ] Unit tests cover core logic (dedup, math, parsing).
*   [ ] `cargo clippy` is silent.
*   [ ] No new `unwrap()` calls in critical paths.

---

## 📅 Phased Execution Plan

### Phase 0: Foundation Fixes (Immediate)
**Goal:** Stabilize the existing Rust skeleton and fix data integrity.

- [ ] **Fix Timestamps (Critical):** Pass `SystemTime` from capture -> dedup -> storage. Remove `Utc::now()` in storage layer.
- [ ] **Config System:** Implement `config` crate. Load `recall.toml`. Remove hardcoded `DEDUP_THRESHOLD`.
- [ ] **Logging:** standardise `tracing` output (JSON for files, pretty for CLI).
- [ ] **Local CI:** Create `justfile` recipes for Tier 1-3 testing.

### Phase 1: The Lazy Workers (Porting Processors)
**Goal:** Move heavy lifting (OCR, Vision) out of Python and into async Rust workers.
*Reference: `docs/plans/mirix-agent-patterns.md`*

- [ ] **OCR Worker:**
    - [ ] Implement Poll-Loop (Postgres `SKIP LOCKED`).
    - [ ] Integrate `leptess` (Tesseract) or calling external binary.
    - [ ] Update `frames` table status columns.
- [ ] **Vision Worker:**
    - [ ] Implement Poll-Loop (Vision Status).
    - [ ] Integrate `reqwest` for LLM API (OpenAI/Anthropic compatible).
    - [ ] Implement Rate Limiting (Token bucket or simple sleep).

### Phase 2: Memory & Hierarchy
**Goal:** Implement the aggregation logic and vector storage.

- [ ] **Vector Storage:** Implement `pgvector` insertion in Rust.
- [ ] **Session Consolidator:**
    - [ ] Implement gap-detection logic (300s threshold).
    - [ ] Create `Activity` rows.
- [ ] **Hierarchy Rollup:**
    - [ ] `Activity` -> `Project` summarization logic.
    - [ ] `Project` -> `DaySummary` aggregation.

### Phase 3: Agent Parity (The Long Tail)
**Goal:** Replace the complex Python agents with Rust traits.

- [ ] **Memory Traits:** Define `MemoryStore` trait (Insert, Query, Update).
- [ ] **Implement Stores:** Port `Episodic`, `Semantic`, `Procedural` logic to Rust structs.
- [ ] **Orchestrator:** Implement the "Meta Memory" logic (routing updates) in Rust.

---

## 📊 Status Tracking

| Phase | Status | Completion % |
| :--- | :--- | :--- |
| **0. Foundation** | 🟡 In Progress | 10% |
| **1. Lazy Workers** | 🔴 Pending | 0% |
| **2. Memory** | 🔴 Pending | 0% |
| **3. Agents** | 🔴 Pending | 0% |
