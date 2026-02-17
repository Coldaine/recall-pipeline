---
doc_type: scratchpad
subsystem: general
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# Agent Working Scratchpad: Critical Refactor Session

> **HISTORICAL DOCUMENT** - This scratchpad is from a previous session (2025-11-25). Many items here have been superseded:
> - `recall_pipeline/` directory was deleted
> - Architecture simplified to server-first Postgres (no local SQLite/DuckDB)
> - Audio capture is out of scope
> - Schema unification completed (see `docs/plans/schema-unification-plan.md`)
>
> Preserved for context on how decisions evolved.

**Agent**: Claude Opus 4.5 (claude-opus-4-5-20251101)
**Session Date**: 2025-11-25
**Time**: Historical (session completed)
**User**: Patrick (PATRICK-DESKTOP)
**Branch**: `hybrid-agent-impl` (superseded)
**Explicit User Instruction**: User explicitly instructed me to create this scratchpad, document my understanding, sign it, and date it. This is not agent-initiated documentation.

---

## What This System Actually Is

**Recall Pipeline is a personal memory system for your digital life.**

It continuously watches screen activity, extracts meaning from it, and stores it in a way that lets the user query their own past. It's a searchable, queryable record of everything done on the computer.

### The Core Flow (6 Capabilities)

| # | Capability | Description |
|---|------------|-------------|
| 1 | **Capture** | High-performance screen recording (frames, not video). Rust-based, Windows 11 first. |
| 2 | **Extract** | OCR pulls text from what's visible. Perceptual hashing deduplicates redundant frames. |
| 3 | **Store** | PostgreSQL holds data with full-text search (BM25) and semantic search (pgvector embeddings). |
| 4 | **Consolidate** | Memory agents process raw captures into organized, meaningful memory structures. |
| 5 | **Query** | User asks questions: "what was I working on Tuesday?", "where did I see that error?", "what was in that document?" |
| 6 | **Recall** | System retrieves relevant moments from digital history and returns them to the user. |

### The 6 Memory Agent Types

These represent different modes of memory organization and retrieval:

| Agent | Purpose |
|-------|---------|
| **Core Memory** | Persistent facts about the user and their work |
| **Episodic Memory** | Specific moments in time ("that meeting on Tuesday") |
| **Semantic Memory** | Conceptual relationships ("projects related to X") |
| **Procedural Memory** | Workflows, how the user does things |
| **Resource Memory** | Files, links, assets encountered |
| **Knowledge Vault** | Consolidated long-term knowledge |

### Why Multi-Deployment Matters (Secondary Concern)

The user works across 3 machines (2 laptops, 1 workstation). The system must:
- Capture from any machine
- Store to a unified memory
- Query from any machine
- Route results back to the active deployment

This is infrastructure supporting the core purpose, not the purpose itself.

---

## Source Project Integration Status

This project was built by extracting from TWO existing open-source projects:

### 1. MIRIX (Python multi-agent memory system)
**Status: INTEGRATED - Work complete**

The 6 memory agents and Python orchestration layer have been successfully extracted from MIRIX and integrated into recall-pipeline. This work is done.

### 2. screenpipe (Rust screen capture engine)
**Status: STILL EMBEDDED - The remaining problem**

screenpipe still exists as a full subdirectory in the repo with all its original baggage:
- `screenpipe-core/` - capture engine (NEEDED)
- `screenpipe-vision/` - OCR/vision processing (NEEDED)
- `screenpipe-app-tauri/` - full Tauri desktop app (NOT NEEDED)
- `pipes/` - dozens of example applications (NOT NEEDED, being deleted)
- `target/` - build artifacts (NOT NEEDED)
- onnxruntime binaries (questionable)
- Their documentation, CI, configs (NOT NEEDED)

**What we need from screenpipe:**
- Rust capture engine for Capture capability
- OCR/vision processing for Extract capability
- FFI bindings (Tesseract, possibly others)

**What we don't need:**
- Their desktop app
- Their example pipes
- Their build artifacts
- Their docs/CI
- Anything not directly serving Capture or Extract

### The Extraction Task

1. Identify which screenpipe modules map to Capture and Extract
2. Extract just those into recall-pipeline's own structure
3. Delete the entire screenpipe subdirectory
4. Document OUR system, not screenpipe's system

**Open question:** What recall-pipeline code already exists vs what is still living inside screenpipe?

---

## Subagent Exploration Results (2025-11-25)

### Finding 1: Two SEPARATE Capture Systems Exist

**This is critical:** There are TWO parallel, non-integrated capture implementations:

| System | Location | Language | Capture Method | OCR | Storage |
|--------|----------|----------|----------------|-----|---------|
| **Python Capture** | `recall_pipeline/` | Python | windows-capture package | PaddleOCR | SQLite + Chroma |
| **Rust Capture** | `screenpipe/screenpipe-server/` | Rust | xcap crate | Tesseract / Windows OCR API | PostgreSQL |

**They do NOT talk to each other.** No FFI, no HTTP calls, no subprocess spawning. The only shared point is the database schema concept.

### Finding 2: recall-pipeline Owned Code

**`recall_pipeline/`** (18 Python files) - Core capture pipeline:
- `capture_win.py` - Windows screen capture via windows-capture
- `capture_linux.py` - Linux capture via XLib/PulseAudio
- `capture_manager.py` - Platform-agnostic orchestration
- `pipeline.py` - capture → OCR → dedup → embed → classify → store
- `ocr.py` - PaddleOCR integration
- `embed.py` - OpenAI/Voyage embeddings
- `store.py` - SQLite storage
- `vectordb.py` - Chroma with monthly rotation
- `dedup.py` - Perceptual hashing (phash)
- `cli.py` - Main entry point

**`agents/`** (188 Python files) - MIRIX integrated memory system:
- 7 specialized memory agents (Core, Episodic, Semantic, Procedural, Meta, Resource, Knowledge Vault)
- Multi-provider LLM support (OpenAI, Anthropic, Google, Azure, Cohere, Mistral)
- FastAPI REST server
- SQLAlchemy ORM for 15+ data models
- MCP tool execution

**`scripts/`** - Utility scripts (DB migration, service installation)

**NO Rust code exists outside screenpipe/** - The architecture document says "Rust for capture" but currently Python handles capture too.

### Finding 3: screenpipe Directory Contents

**Total size: ~15GB** (mostly deletable)

| Directory | Size | Needed? | Purpose |
|-----------|------|---------|---------|
| `target/` | 12GB | NO - DELETE | Build artifacts |
| `screenpipe-app-tauri/` | 1.1GB | NO - DELETE | Desktop GUI app |
| `screenpipe-js/` | 1.3MB | NO - DELETE | JavaScript SDK |
| `pipes/` | 1.9MB | NO - DELETE | Example applications |
| `content/` | 2.6MB | NO - DELETE | Documentation images |
| `screenpipe-core/` | 438KB | YES | Platform APIs, FFmpeg, utilities |
| `screenpipe-vision/` | 1.4MB | YES | Screenshot capture, OCR |
| `screenpipe-audio/` | 337MB | NO - OUT OF SCOPE | Audio capture (not needed for screen recall) |
| `screenpipe-server/` | 546KB | YES | HTTP API, orchestration |
| `screenpipe-db/` | 303KB | YES | Database abstraction |
| `screenpipe-events/` | 47KB | YES | Event bus |
| `screenpipe-storage/` | 100KB | YES | Storage abstraction |
| `screenpipe-integrations/` | 53KB | MAYBE | External APIs, MCP |
| `screenpipe-memory/` | 27KB | MAYBE | LLM context |
| `screenpipe-llm/` | 52KB | MAYBE | LLM utilities |

### Finding 4: Integration Status - CORRECTED

**The hybrid architecture IS implemented.** I was wrong in my initial assessment.

**Integration method: Database as queue**

```
┌─────────────────────────────────────────────────────────────────────┐
│                    HYBRID ARCHITECTURE (WORKING)                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  RUST HOT PATH (recall-rs binary)                                   │
│  ├─ Screen capture via xcap                                         │
│  ├─ Deduplication via phash64                                       │
│  ├─ Save frames to PostgreSQL (vision_status = 0)                   │
│  ├─ OCR worker (Tesseract) - background task                        │
│  └─ Window context worker - background task                         │
│                          │                                          │
│                          ▼                                          │
│                    ┌──────────┐                                     │
│                    │ DATABASE │ ← Integration Layer                 │
│                    │(PostgreSQL)│                                   │
│                    └──────────┘                                     │
│                          │                                          │
│                          ▼                                          │
│  PYTHON AGENTS (agents/processors/frame_processor.py)               │
│  ├─ Poll for frames where vision_status = 0                         │
│  ├─ Process with VisionAgent (LLM summarization)                    │
│  ├─ Update vision_status = 1 and vision_summary                     │
│  └─ Memory agents (episodic, semantic, etc.) consume summaries      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**This IS correct integration.** Database-as-queue is a valid architectural pattern:
- No FFI complexity
- No HTTP latency
- Clean separation of concerns
- Either side can be replaced independently

### Finding 5: The Three Codebases

| Codebase | Location | Status | Purpose |
|----------|----------|--------|---------|
| **recall_pipeline/** | `recall_pipeline/` | LEGACY - Third codebase | Original Python-only implementation |
| **screenpipe** | `screenpipe/` | ACTIVE - Extracted | Rust hot path (capture, OCR, storage) |
| **MIRIX** | `agents/` | INTEGRATED | Python memory agents |

**What the user described is accurate:**
- screenpipe → source for Rust hot path (being extracted)
- MIRIX → source for Python agents (already integrated into `agents/`)
- recall_pipeline → their ORIGINAL codebase (legacy, to be deprecated?)

### Finding 6: What Actually Works

**Rust side (`recall-rs`):**
- `screenpipe-storage` - Storage trait with LocalStorage and PgStorage
- `screenpipe-vision` - phash64 for deduplication
- `screenpipe-server/workers` - OCR worker (Tesseract), window context worker
- Binary captures frames, dedupes, saves to DB, spawns workers

**Python side (`agents/`):**
- `frame_processor.py` - Polls DB for pending frames, calls VisionAgent
- 7 memory agents from MIRIX
- FastAPI server for agent orchestration

### The Remaining Questions

1. Is `recall_pipeline/` still being used, or is it fully deprecated in favor of Rust capture?
2. What's the status of the Python capture code (`recall_pipeline/capture_win.py`) vs Rust capture (`recall-rs`)?
3. Can we delete `recall_pipeline/` entirely, or is there value to preserve?

---

## User's Core Problem Statement

The codebase has accumulated:
- Dozens of branches with unclear purposes
- Dozens of PRs with conflicting directions
- Documentation scattered across multiple locations
- CI configurations with unknown rationale
- Agent-to-agent handoffs that lose context and create ambiguity

The user wants to **stop all ambiguity** and establish a single source of truth.

---

## Critical Architectural Requirement: Distributed Deployment

The user has explicitly confirmed the following architectural constraint:

### Single-User, Multi-Deployment Model

This is **NOT** a multi-user system. It is a **single user operating across multiple deployment endpoints**:

| Machine | Role |
|---------|------|
| Laptop 1 | Deployment endpoint |
| Laptop 2 | Deployment endpoint |
| Primary Workstation | Deployment endpoint (likely primary) |

### Required Capabilities

1. **Active Deployment Detection**: System must detect which deployment endpoint is currently active
2. **Dynamic Output Redirection**: Route processing output to any of the three machines based on:
   - Which machine initiated the process
   - Current context
   - Resource availability
   - Operational requirements
3. **Flexible Routing**: Results can be sent to:
   - The originating laptop
   - The primary workstation
   - A designated processing node
4. **Deployment Context Awareness**: Runs tagged with `deployment_id` + `agent_run_id`

### Key Distinction to Preserve

| Concept | Definition | System Support |
|---------|------------|----------------|
| **Multi-deployment** | Single user across multiple machines | YES - core requirement |
| **Multi-user** | Multiple users on shared infrastructure | NO - not a goal |

This distinction must be preserved and documented clearly in the architecture.

---

## Current State Assessment

### Branch Chaos (Confirmed)
```
Local branches:
  claude/evaluate-existing-solutions-*
  feat/ffi-specs
  fix/ci-obsolete-tests
  hybrid-agent-impl  <-- CURRENT
  main
  pr-5
  restore/hybrid-agents-docs
  revert/confirm-pr-5

Remote branches (partial):
  origin/claude/* (multiple)
  origin/docs-hybrid-architecture
  origin/feat/*
  origin/fix/*
  origin/pure-rust-migration-phase-1
  origin/restore/*
  origin/revert/*
```

This confirms the user's description of chaos.

### Current Working Tree State

**Staged deletions** (files removed from main):
- `AGENTS.md` - old agentic instructions
- `docs/ARCHITECTURE.md` - detailed but scattered architecture
- `docs/DECISIONS.md` - ADR log
- `docs/HYBRID_ARCHITECTURE.md` - hybrid approach docs
- `docs/IMPLEMENTATION_PLAN.md` - old plans
- `docs/MIGRATE_MILVUS.md` - migration docs
- `docs/rust/*` - Rust-specific guidelines
- Various plan files at root
- Screenpipe example pipes (hundreds of files)
- Old CI workflows (`rust-ci.yml`, `rustdoc.yml`)

**New structure created** (exists but skeletal):
```
/docs/
├── README.md                    # Points to this as single source of truth
├── architecture/
│   └── overview.md              # High-level, mentions deployment model
├── domains/
│   ├── capture.md
│   ├── storage.md
│   ├── orchestration.md
│   └── routing.md
├── dev/
│   ├── ci.md                    # "Rebuild from scratch" stub
│   ├── logging.md
│   └── testing.md
└── project-management/
    ├── changelog.md             # Guidelines stub
    ├── worklog.md               # Guidelines stub
    └── agent-playbook.md        # Basic rules
```

**CI State**: Placeholder only (`workflow_dispatch`, no actual checks)

### Value in Old Docs (To Extract)

From `main:docs/ARCHITECTURE.md`:
- MIRIX 6-agent memory system design
- Rust capture + Python orchestration split rationale
- PostgreSQL + pgvector integration details
- Component flow diagrams
- Technology stack decisions

This content should be synthesized into the new structure, not lost.

---

## My Execution Plan

### Phase 1: Extract Value from Old Docs

I will read all deleted docs from main branch and extract:
- Architectural decisions worth preserving
- Technology stack rationale
- Component diagrams and data flows
- The 6-agent memory system design
- PostgreSQL/pgvector integration details

### Phase 2: Populate New Docs Structure

**`/docs/architecture/overview.md`** - Expand to include:
- Full future state architecture
- Single-user, multi-deployment model (explicitly)
- Data plane flow with deployment awareness
- Control plane with routing configuration
- Component diagram with deployment endpoints

**`/docs/domains/`** - Fill each with substance:
- `capture.md` - Rust capture engine, Windows 11, pHash deduplication
- `storage.md` - DuckDB local / Postgres+Timescale+pgvector, deployment tagging
- `orchestration.md` - Python agents, MIRIX integration, 6-agent design
- `routing.md` - Dynamic output redirection, deployment detection, endpoint selection

**`/docs/dev/`** - Actionable practices:
- `ci.md` - Actual CI configuration with doc-check gates
- `logging.md` - Structured tracing requirements
- `testing.md` - Unit/integration/load test expectations

**`/docs/project-management/`** - Control docs:
- `worklog.md` - Active work log template
- `changelog.md` - Proper changelog format
- `agent-playbook.md` - Mandatory reading list, contribution rules

### Phase 3: Create Agentic Control Files

**`/CLAUDE.md`** (root):
- Mandatory pointer to `/docs/project-management/`
- Brief project context
- Key constraints (single-user multi-deployment)
- "Read these docs before coding" mandate

**`/AGENTS.md`** (root):
- Same mandatory reading requirements
- Rules for any AI agent working on this codebase
- Documentation update requirements

### Phase 4: Rebuild CI from Scratch

**`/.github/workflows/ci.yml`** - Replace placeholder with:
- Rust lint/build smoke test
- Python lint/test
- Doc-check gate: fail if `/docs/` files not updated with scope changes
- Optional: Agentic enforcement via Claude/Gemini CLI

### Phase 5: Commit as Atomic Change

- All changes in one commit
- No incremental PRs
- No agent-to-agent handoffs
- Clean break from old state

---

## What I Will NOT Do

1. **Archive anything** - User explicitly stated archives confuse agents
2. **Create incremental handoffs** - Everything in one pass
3. **Preserve old CI** - Rationale unknown, rebuild from scratch
4. **Leave stubs** - Content must be substantive, not placeholders
5. **Guess at requirements** - Documented constraints only

---

## Constraints I Will Enforce

1. Single-user, multi-deployment architecture preserved
2. Rust hot path / Python orchestration split maintained
3. Deployment ID + agent run ID tagging required
4. Dynamic output redirection capability preserved
5. No multi-user tenancy assumptions introduced

---

## Sign-Off

I, Claude Opus 4.5, confirm:
- I have read the user's instructions
- I understand the distributed deployment requirement
- I understand the documentation architecture requirement
- I will execute this plan in one atomic pass
- I will not archive, handoff incrementally, or leave stubs
- The user explicitly instructed me to create this scratchpad

**Awaiting user confirmation to proceed with execution.**

---

*This scratchpad will be updated as work progresses. It exists per user instruction as a working memory for this session.*
