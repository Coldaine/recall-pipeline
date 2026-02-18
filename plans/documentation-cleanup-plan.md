---
last_edited: 2026-02-18
editor: Kilo Code (Architect Mode)
user: Coldaine
status: draft
version: 1.0.0
subsystem: general
tags: [documentation, cleanup, restructuring, plan]
doc_type: plan
---

# Documentation Cleanup Plan

> **Goal**: Prune superfluous documentation and establish a clean, minimal doc tree that supports the Pure Rust architecture (ADR-009).

---

## 1. Current State Analysis

### 1.1 Files to Delete (Superfluous)

| Path | Reason |
|------|--------|
| `docs/session-logs/` | Session logs are ephemeral, not architecture |
| `docs/scratchpad/` | Working notes, not canonical documentation |
| `docs/plans/consolidation-2025-11.md` | Historical, no longer actionable |
| `docs/plans/mirix-agent-patterns.md` | **MOVE** to `docs/architecture/` (it's reference, not a plan) |
| `docs/revision_log.csv` | Empty file |

### 1.2 Archive Files to Prune

| Path | Reason |
|------|--------|
| `docs/archive/dev/ci.md` | Superseded by `docs/dev/testing.md` |
| `docs/archive/dev/testing.md` | Superseded by `docs/dev/testing.md` |
| `docs/archive/dev/logging.md` | Minimal content (383 chars), not useful |
| `docs/archive/dev/postgres-extensions.md` | Should be in `docs/storage/` if needed |
| `docs/archive/project-management/` | All 3 files are historical |
| `docs/archive/architectural-docs-discovery.md` | Discovery doc, no longer needed |
| `docs/archive/code-inventory.md` | Stale inventory |
| `docs/archive/ffi-bridge.md` | FFI bridge abandoned per ADR-009 |
| `docs/archive/multi-agent-review.md` | Review complete, patterns extracted |
| `docs/archive/pr-review-agents.md` | Review complete |
| `docs/archive/screenpipe-crate-audit.md` | Audit complete, crates renamed |

### 1.3 Archive Files to Keep

| Path | Reason |
|------|--------|
| `docs/archive/direct-quotes-and-vision.md` | User's voice, valuable context |
| `docs/archive/foundational-principles.md` | Historical reference |
| `docs/archive/unified-vision.md` | Vision context |

---

## 2. Desired Documentation Tree

```
docs/
|-- index.md                    # Architecture hub (KEEP)
|-- Northstar.md                # Vision & principles (KEEP)
|-- roadmap.md                  # Implementation phases (KEEP)
|-- todo.md                     # Task backlog (KEEP)
|-- MasterDocumentationPlaybook.md  # Governance (KEEP)
|
|-- architecture/               # Architectural decisions
|   |-- adr-001.md              # [NEW] Project scope & single-user model
|   |-- adr-002.md              # [NEW] Lazy processing pattern
|   |-- adr-003.md              # PostgreSQL Only (KEEP)
|   |-- adr-004.md              # [NEW] Hierarchical summarization
|   |-- adr-005.md              # MIRIX Agents (KEEP)
|   |-- adr-006.md              # Windows First (KEEP)
|   |-- adr-007.md              # [NEW] FIDO2 encryption for secrets
|   |-- adr-008.md              # Gradual Extraction (KEEP)
|   |-- adr-009.md              # Pure Rust (KEEP)
|   |-- mirix-agent-patterns.md # [MOVE from plans/]
|
|-- capture/                    # Capture domain
|   |-- cp-overview.md          # (KEEP)
|
|-- storage/                    # Storage domain
|   |-- st-overview.md          # (KEEP)
|   |-- schema.md               # [NEW] Full SQL schema reference
|
|-- orchestration/              # Orchestration domain
|   |-- or-overview.md          # (KEEP)
|   |-- or-routing.md           # (KEEP)
|
|-- dev/                        # Developer guides
|   |-- testing.md              # (KEEP)
|   |-- setup.md                # [NEW] Environment setup
|   |-- contributing.md         # [NEW] Contribution guidelines
|
|-- archive/                    # Historical reference only
|   |-- direct-quotes-and-vision.md
|   |-- foundational-principles.md
|   |-- unified-vision.md
```

---

## 3. ADR Consolidation Plan

### 3.1 Current ADR Gaps

Current ADRs are numbered 003, 005, 006, 008, 009. Missing: 001, 002, 004, 007.

### 3.2 Proposed ADR Additions

| ADR | Title | Content Source |
|-----|-------|----------------|
| ADR-001 | Single-User Multi-Deployment Model | Northstar.md §1 |
| ADR-002 | Lazy Processing Pattern | Northstar.md §1, mirix-patterns §2 |
| ADR-004 | Hierarchical Summarization | mirix-patterns §4 |
| ADR-007 | FIDO2 Encryption for Secrets | Northstar.md §1, mirix-patterns §1.5 |

### 3.3 ADR Template

```markdown
---
last_edited: YYYY-MM-DD
editor: Name
user: Coldaine
status: active
version: 1.0.0
subsystem: architecture
tags: [adr, decision]
doc_type: adr
---

# ADR-NNN: Title

## Status
Active | Superseded | Deprecated

## Context
What is the issue we're addressing?

## Decision
What have we decided to do?

## Consequences
- **Positive**: Benefits
- **Negative**: Trade-offs
```

---

## 4. Root-Level Agent Files

### 4.1 AGENTS.md (Update Required)

Current: Points to legacy Python architecture
Desired: Point to current Rust architecture and doc tree

```markdown
# Recall Pipeline - Agent Instructions

## Project Summary
Total digital recall system. Capture screen activity, deduplicate, OCR, summarize with LLMs.

**Single-user, multi-deployment model** - one person, many machines, central aggregation.

## Tech Stack
| Layer | Tech | Location |
|-------|------|----------|
| Capture | Rust | `capture/` (3 crates) |
| Storage | Postgres + pgvector | Central server |
| Processing | Rust workers | Lazy OCR/Vision/Embedding |

## Key Documentation
- [Northstar](docs/Northstar.md) - Vision & principles
- [Roadmap](docs/roadmap.md) - Implementation phases
- [Architecture](docs/architecture/) - ADRs and patterns

## Quick Start
```bash
cd capture && cargo build --workspace
just ci
```

## Code Conventions
- Rust: `anyhow` for errors, Tokio async, feature flags
- Status codes: 0=Pending, 1=Done, 2=Error, 3=Skipped
```

### 4.2 CLAUDE.md (Update Required)

Should reference the same documentation tree and be consistent with AGENTS.md.

---

## 5. Implementation Checklist

### Phase 1: Delete Superfluous Files
- [ ] Delete `docs/session-logs/` directory
- [ ] Delete `docs/scratchpad/` directory
- [ ] Delete `docs/plans/consolidation-2025-11.md`
- [ ] Delete `docs/revision_log.csv`

### Phase 2: Prune Archive
- [ ] Delete 11 archive files listed in §1.2
- [ ] Keep 3 archive files listed in §1.3

### Phase 3: Move and Create
- [ ] Move `docs/plans/mirix-agent-patterns.md` to `docs/architecture/`
- [ ] Create `docs/storage/schema.md` with full SQL schema
- [ ] Create `docs/dev/setup.md` with environment setup
- [ ] Create `docs/dev/contributing.md` with guidelines

### Phase 4: ADR Consolidation
- [ ] Create ADR-001 (Single-User Multi-Deployment)
- [ ] Create ADR-002 (Lazy Processing)
- [ ] Create ADR-004 (Hierarchical Summarization)
- [ ] Create ADR-007 (FIDO2 Encryption)

### Phase 5: Update Root Files
- [ ] Update `AGENTS.md` to reflect Rust architecture
- [ ] Update `CLAUDE.md` to match AGENTS.md
- [ ] Update `docs/index.md` to reflect new tree

---

## 6. File Count Summary

| Category | Before | After |
|----------|--------|-------|
| Active docs | 32 | 18 |
| Archive docs | 14 | 3 |
| Deleted | 0 | 25 |
| **Total** | 46 | 21 |

---

*Plan created 2026-02-18 by Kilo Code Architect Mode*