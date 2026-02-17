---
doc_type: architecture
subsystem: general
version: 2.0.0
status: active
owners: Patrick
last_reviewed: 2026-02-09
---
# Code Inventory

This document tracks what code exists, its status, and whether to use it.

## Production Ready — Use These

| Location | What | Notes |
|----------|------|-------|
| `capture/recall-capture/` | Screen capture + dedup | xcap 0.8, phash64, histogram/SSIM |
| `capture/recall-db/` | Postgres database | sqlx, TimescaleDB migrations |
| `capture/recall-store/` | Storage abstraction | Storage trait, PgStorage, ImageStorage |
| `capture/src/bin/recall.rs` | Capture daemon binary | Multi-monitor, CLI, daily cleanup |
| `agents/llm_api/` | LLM clients | OpenAI, Anthropic, Google, Azure — working |
| `agents/prompts/` | Agent prompts | Prompt templates |

## Exists But Not Yet Wired

| Location | What | Notes |
|----------|------|-------|
| `agents/processors/` | Frame processor pattern | Needs adaptation for new schema |
| `agents/agent/vision.py` | Vision summarization | LLM vision agent |
| `agents/orm/activity.py`, `project.py`, `day_summary.py` | Hierarchy ORM | Schema-unification branch |
| `agents/orm/secret.py` | Secrets ORM | Unencrypted, FIDO2 pending |

## Experimental — Deprioritized (MIRIX)

| Location | What | Notes |
|----------|------|-------|
| `agents/agent/*_memory_agent.py` | Memory agents | 6 types, still uses org/user schema |
| `agents/orm/*_memory.py` | Memory ORM | Needs deployment_id migration |
| `agents/services/*_memory_manager.py` | Memory services | Service layer |

## Stubs / Incomplete

| Location | What | Notes |
|----------|------|-------|
| `agents/llm_api/cohere*.py` | Cohere client | Stub |
| `agents/llm_api/mistral.py` | Mistral client | Partial |
| `agents/llm_api/aws_bedrock.py` | Bedrock client | Stub |

## Removed

All screenpipe-\* crates removed 2026-02-09:

screenpipe-audio, screenpipe-core, screenpipe-db, screenpipe-events, screenpipe-integrations, screenpipe-llm, screenpipe-memory, screenpipe-server, screenpipe-storage, screenpipe-ui, screenpipe-vision.

Replaced by recall-capture, recall-db, recall-store.

## Related

- Implementation phases: `docs/plans/implementation-roadmap.md`
- Architecture overview: `docs/architecture/overview.md`
- Storage domain: `docs/domains/storage.md`
