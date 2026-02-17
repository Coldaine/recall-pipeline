---
last_edited: 2026-02-14
editor: Antigravity (Gemini)
user: Coldaine
status: ready
version: 1.1.0
subsystem: monorepo
tags: [principles, strategy, northstar]
doc_type: architecture
---

# North Star Principles


> **"Total digital recall + perfect AI context. Capture everything across all machines, summarize intelligently at multiple levels, make it searchable and usable as context for LLMs."**

This document describes the immutable **North Star** for the Recall Pipeline. These principles are non-negotiable.

## 1. Foundational Principles

- **Single-User, Multi-Deployment**: One person, multiple machines. Data flows to a central server. **No multi-user tenancy.**
- **PostgreSQL Only**: No SQLite. SQLite died from write contention. Single Postgres instance is the source of truth. (See [ADR-001](architecture/adr-001.md) for the full trade-off analysis.)
- **Rust for Core; Python for Agents**: Capture and storage are Pure Rust. Agents (MIRIX) are first-class Python. No Python in the capture hot path. (See [ADR-002](architecture/adr-002.md) for details.)
- **Lazy Processing**: Never block capture. OCR, vision, and summarization happen asynchronously on the server, or potentially locally (PHASE 2. ).
- **Hierarchical Summaries**: Drill down: Day → Project → Activity → Frame.
- **Secure by Default**: Secrets must be redacted and stored encrypted (requiring FIDO2 key to decrypt).
- **Windows 11 First**: The MVP targets Windows 11 exclusively. Mac/Linux support is deferred. (Formerly ADR-006.)


## 2. Direct Quotes (The User's Voice)

> "The scope of this is large. I want to both recall, and give perfect context."

> "Ultimately a hierarchy: Local caching/storage gets uploaded... or stored wholesale on my remote aggregation server. Key to this vision is the 'lazy review' ... certainly not realtime."

> "I want to be able to visualize in dashboards what I was doing at each part of the day, with intelligent summarization... aggregat[ing] that into multiple levels that I can drill down into or up from."

> "I have a FIDO2 key that I want all sensitive information to be encrypted with."

> "Safety > speed for architectural refactor."

## 3. Anti-Patterns (Never Do This)

❌ **Python in the Hot Path**: Slows down capture.
❌ **SQLite or Local-First**: Proven to fail under write load.
❌ **Real-Time Processing on Device**: Too heavy for laptops; must be lazy.
❌ **Tight IPC Coupling**: Brittle; use DB as the boundary.
❌ **Multi-User Tenancy**: Scope creep; single user only.
❌ **Inventing New Agent Architectures**: MIRIX exists; use it.
❌ **Big-Bang Rewrites**: Extract gradually, one component at a time.

## 4. Architecture Decision Records

Only decisions with significant, contested trade-offs get a full ADR:

- [ADR-001: PostgreSQL Only](architecture/adr-001.md) — Rejected SQLite due to write contention.
- [ADR-002: Pure Rust Stack](architecture/adr-002.md) — Rejected hybrid Python/Rust in production.
- [ADR-003: Capture Rewrite from Screenpipe](architecture/adr-003.md) — Port upstream capture code rather than clean-room rewrite.

## 5. Storage Schema Commandments

1.  **`deployment_id`** on every table.
2.  **No `user_id`**.
3.  **Status Codes**: `0=Pending, 1=Done, 2=Error, 3=Skipped`.
4.  **Hierarchical Tables**: `frames` → `activities` → `projects` → `day_summaries`.
5.  **Secrets**: Stored separately, encrypted.
