# Agents (Legacy Reference)

> **⚠️ STATUS WARNING**: This document describes the **Legacy Python Architecture** (`agents/`).
>
> As per **ADR-009**, the system is moving to a **Pure Rust** architecture.
>
> *   For the target architecture patterns, see: [`docs/plans/mirix-agent-patterns.md`](docs/plans/mirix-agent-patterns.md)
> *   For the implementation plan, see: [`docs/plans/implementation-roadmap.md`](docs/plans/implementation-roadmap.md)
> *   For the current Rust codebase status, check `capture/`.

---

## Legacy Python System Summary

The `agents/` directory contains a Python 3.12+ implementation of a MIRIX-derived memory system.

### Tech Stack (Deprecated)
- **Framework**: FastAPI
- **LLM**: LlamaIndex + OpenAI/Anthropic
- **OCR**: PaddleOCR (Heavy dependency)
- **Database**: SQLAlchemy (Sync) + asyncpg (Async)

### Agent Types (To be ported to Rust)

| Agent | Purpose | Rust Port Status |
|-------|---------|------------------|
| **Episodic** | Event timeline | 🔴 Pending |
| **Semantic** | Facts & knowledge | 🔴 Pending |
| **Procedural** | Workflows | 🔴 Pending |
| **Resource** | Files & Links | 🔴 Pending |
| **Knowledge Vault** | Secrets | 🔴 Pending |
| **Core** | Identity | 🔴 Pending |
| **Meta** | Routing/Orchestration | 🔴 Pending |

### Directory Structure (Legacy)

```
agents/
  agent/          # The Agent logic (Python classes)
  llm_api/        # LLM Clients
  orm/            # Database Models (SQLAlchemy)
  processors/     # OCR/Vision Workers (Python)
```
