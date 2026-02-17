---
last_edited: 2026-02-13
editor: Claude Code (Claude Opus 4.5)
user: Coldaine
status: active
version: 1.0.0
subsystem: architecture
tags: [index, vision, roadmap]
doc_type: architecture
---

# Architecture Overview

**"Total digital recall + perfect AI context."**

This document serves as the high-level entry point for the system architecture.

## 🧭 Core Documentation

- **[North Star (Vision & Principles)](Northstar.md)**: The immutable "Why" and "How". Read this first.
- **[Implementation Roadmap](roadmap.md)**: The active plan (6 Phases).
- **[Task Backlog](todo.md)**: Single source of truth for active tasks.
- **[Documentation Playbook](MasterDocumentationPlaybook.md)**: Governance.

## 🏗️ key Decisions (ADRs)

- **[ADR-003](architecture/adr-003.md)**: PostgreSQL Only (No SQLite)
- **[ADR-009](architecture/adr-009.md)**: Pure Rust End-to-End
- **[ADR-005](architecture/adr-005.md)**: MIRIX Agents
- **[ADR-006](architecture/adr-006.md)**: Windows First
- **[ADR-008](architecture/adr-008.md)**: Gradual Extraction

## 🏢 Domains

- **[Storage](storage/st-overview.md)**: Schema, config.
- **[Capture](capture/cp-overview.md)**: Rust capture crate.
- **[Orchestration](orchestration/or-overview.md)**: Workers & Routing.

## 📋 Plans

- **[MIRIX Agent Patterns](plans/mirix-agent-patterns.md)**: Extracted architectural patterns from `agents/` (pre-deletion reference)
- **[Schema Unification](plans/schema-unification-plan.md)**: Schema unification plan
- **[Consolidation 2025-11](plans/consolidation-2025-11.md)**: November 2025 consolidation

## 🛠 Developer
- **[Testing Strategy](dev/testing.md)**: Categories, runner commands, CI.
- **[Documentation Playbook](MasterDocumentationPlaybook.md)**: Governance.

## 📐 System at a Glance

```
DEPLOYMENTS (Clients)                    SERVER (Central)
┌──────────────────────┐                ┌──────────────────────────┐
│ Laptop / Desktop     │   HTTP POST    │ Postgres + pgvector      │
│  - Rust Capture      │ ─────────────▶ │  - frames, summaries     │
│  - Phash Dedup       │                │                          │
│  (No Local DB)       │                │ Rust Workers (Lazy)      │
└──────────────────────┘                │  - OCR, Vision, Summary  │
                                        └───────────────────────────┘
```

See [Northstar.md](architecture/Northstar.md) for the detailed "Anti-Patterns" and "Schema Commandments".
