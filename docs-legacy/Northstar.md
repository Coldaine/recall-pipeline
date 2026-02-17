---
last_edited: 2026-02-13
editor: Claude Code (Claude Opus 4.5)
user: Coldaine
status: ready
version: 1.0.0
subsystem: general
tags: [vision, principles, northstar, architecture]
doc_type: architecture
---

# North Star: Foundational Principles & Vision

> **"Total digital recall + perfect AI context. Capture everything across all machines, summarize intelligently at multiple levels, make it searchable and usable as context for LLMs."**

This document describes the immutable **North Star** for the Recall Pipeline. These principles are non-negotiable.

## 1. Foundational Principles (The "Above All Else" Rules)

- **Single-User, Multi-Deployment**: One person, multiple machines (laptop, desktop). Data flows to a central server. **No multi-user tenancy.**
- **PostgreSQL Only**: No SQLite. SQLite died from write contention. Single Postgres instance is the source of truth.
- **Pure Rust End-to-End**: (ADR-009) No Python in the production runtime. Rust for capture, storage, and logic. Python is for reference/prototyping only.
- **Lazy Processing**: Never block capture. OCR, vision, and summarization happen asynchronously on the server.
- **Hierarchical Summaries**: Drill down: Day → Project → Activity → Frame.
- **Secure by Default**: Secrets must be redacted and stored encrypted (requiring FIDO2 key to decrypt).

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

## 4. Architecture Decision Records (ADR Summary)

- **ADR-003**: PostgreSQL only (No SQLite).
- **ADR-005**: MIRIX Multi-Agent System (reuse, don't invent).
- **ADR-006**: Windows 11 First (MVP).
- **ADR-008**: Gradual Extraction (Safety > Speed).
- **ADR-009**: Pure Rust Stack (Supersedes Python orchestration).

## 5. Storage Schema Commandments

1.  **`deployment_id`** on every table.
2.  **No `user_id`**.
3.  **Status Codes**: `0=Pending, 1=Done, 2=Error, 3=Skipped`.
4.  **Hierarchical Tables**: `frames` → `activities` → `projects` → `day_summaries`.
5.  **Secrets**: Stored separately, encrypted.
