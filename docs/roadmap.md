---
last_edited: 2026-02-13
editor: Claude Code (Claude Opus 4.5)
user: Coldaine
status: active
version: 2.1.0
subsystem: general
tags: [roadmap, plan, phases]
doc_type: plan
---
# Implementation Roadmap

> See `docs/architecture/overview.md` for the overall vision and architecture.
> See `docs/architecture/code-inventory.md` for what code exists and its status.

## Phase Summary

| Phase | Goal | Status |
|-------|------|--------|
| 1 | Raw capture → Postgres | **Done** |
| 2 | Lazy OCR + vision workers | Next |
| 3 | Hierarchical summaries (activity → project → day) | After Phase 2 |
| 4 | HTTP POST ingestion (multi-machine) | When needed |
| 5 | Memory agents (MIRIX) | Experimental, deprioritized |
| 6 | Secrets & encryption | Future |

---

## Phase 1: Raw Capture → Postgres (Done)

**Goal:** Reliable capture that writes directly to Postgres without dying.

**What shipped:**

| Component | Location | Notes |
|-----------|----------|-------|
| Screen capture | `capture/recall-capture/` | xcap 0.8 direct, full Wayland/PipeWire support |
| Phash dedup | `capture/recall-capture/` | phash64, histogram + SSIM frame comparison |
| Postgres schema | `capture/recall-db/` | TimescaleDB + pgvector, sqlx migrations |
| Storage abstraction | `capture/recall-store/` | Storage trait, PgStorage adapter, ImageStorage (JPEG to disk) |
| Capture daemon | `capture/src/bin/recall.rs` | CLI args, multi-monitor support, daily cleanup |

- `cargo check --workspace` passes clean.
- All screenpipe-* crates removed; replaced by recall-capture, recall-db, recall-store.

**Remaining:** Performance hardening — `spawn_blocking` for JPEG encoding, concurrent monitor capture, channel-based pipeline.

---

## Phase 2: Lazy Processing (Next)

**Goal:** OCR and vision processing on server, async.

| Component | Location | Action |
|-----------|----------|--------|
| Frame processor pattern | `agents/processors/` | Adapt for new schema |
| Vision agent | `agents/agent/vision.py` | LLM vision summarization |
| LLM clients | `agents/llm_api/` | OpenAI, Anthropic, Google, Azure — working |

**What changes:**

- Python OCR worker polls `frames WHERE vision_status = 0`
- Runs Tesseract or LLM vision
- Updates `ocr_text` / `vision_summary`, sets status flags
- Same polling pattern for vision summarization worker

---

## Phase 3: Hierarchical Summaries

**Goal:** Drill-down navigation from day → project → activity → frame.

| Component | Location | Action |
|-----------|----------|--------|
| Summarization prompts | `agents/prompts/` | Extend with activity/project/day prompts |
| Hierarchy ORM | `agents/orm/activity.py`, `project.py`, `day_summary.py` | Tables defined in schema-unification branch |
| LLM orchestration | `agents/services/` | Reuse patterns |

**What changes:**

- Frame clusters → Activity summaries → Project summaries → Day summaries
- Tables `activities`, `projects`, `day_summaries` already defined in Python ORM (schema-unification branch)
- New summarization workers cluster frames into activities, roll up to projects and days

---

## Phase 4: HTTP POST Ingestion (When Needed for Multi-Machine)

**Goal:** Decouple capture machines from direct Postgres access.

**What changes:**

- API server on the Postgres host accepting frame submissions via HTTP POST
- Memory buffer + retry on capture client side for network resilience
- Capture clients POST frames instead of writing to Postgres directly
- Enables laptop/desktop/work PC all feeding the same server

---

## Phase 5: Memory Agents — MIRIX (Experimental, Deprioritized)

**Goal:** Wire existing MIRIX agents to consume from frames and summaries.

| Component | Location | Action |
|-----------|----------|--------|
| Memory agents | `agents/agent/*_memory_agent.py` | Wire to frames/summaries |
| Memory ORM | `agents/orm/*_memory.py` | Needs deployment_id migration (Issue #47) |
| Memory services | `agents/services/*_memory_manager.py` | Adapt to feed from capture data |
| Prompts | `agents/prompts/` | Reuse as-is |

**6 memory types:** Episodic, Semantic, Procedural, Resource, Knowledge Vault, Core.

**Blocker:** MIRIX models still use org/user schema; needs migration to deployment_id (Issue #47).

---

## Phase 6: Secrets & Encryption (Future)

**Goal:** Detect and encrypt sensitive content in captured frames.

**What changes:**

- Secret detection via pattern matching + LLM review
- FIDO2 hardware key encryption for secret storage
- Secrets table exists in schema (Track D of schema-unification)
- ORM model at `agents/orm/secret.py` (unencrypted, FIDO2 pending)

---

## CI Expansion Summary

| Phase | What's tested | New coverage |
|-------|---------------|--------------|
| 1 | Rust workspace check, format, clippy | recall-capture, recall-db, recall-store |
| 2 | OCR worker, vision worker | Processing pipeline |
| 3 | Summarization, hierarchy rollup | Rollup logic |
| 4 | HTTP ingestion endpoint | API server + client retry |
| 5 | Memory agents | Full MIRIX integration |
| 6 | Secret detection, encryption | Secrets pipeline |

**Principle:** CI only tests what's implemented. Don't test MIRIX agents until Phase 5.
