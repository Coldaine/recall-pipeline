# Recall Pipeline - Agent Instructions

## Project Summary

Total digital recall system. Capture screen activity across multiple machines, deduplicate, OCR, summarize with LLMs, and provide searchable context for AI assistants.

**Single-user, multi-deployment model** — one person, many machines, central aggregation.

## Tech Stack

| Layer | Tech | Location |
|-------|------|----------|
| Capture engine | Rust | `capture/` (workspace with 11 crates) |
| Local storage | SQLite/DuckDB | Per-deployment cache |
| Central storage | Postgres + pgvector | Aggregation server |
| Orchestration | Python 3.12+ | `agents/` |
| OCR | Tesseract | Via Rust workers |
| Transcription | Whisper | Via Rust workers |

## Directory Structure

```
capture/                    # Rust workspace
  screenpipe-audio/         # Audio capture, VAD, transcription
  screenpipe-vision/        # Screen capture, OCR
  screenpipe-core/          # Embeddings, LLM integration, utilities
  screenpipe-db/            # Database layer
  screenpipe-storage/       # Storage abstraction
  screenpipe-server/        # HTTP server
  screenpipe-events/        # Event system
  screenpipe-llm/           # LLM budget, vision
  screenpipe-integrations/  # External integrations
  screenpipe-memory/        # Memory features

agents/                     # Python package
  agent/                    # Memory agent implementations
    episodic_memory_agent.py
    semantic_memory_agent.py
    procedural_memory_agent.py
    core_memory_agent.py
    knowledge_vault_agent.py
    resource_memory_agent.py
    meta_memory_agent.py
    reflexion_agent.py
  llm_api/                  # LLM clients (Anthropic, OpenAI, Azure, etc.)
  orm/                      # SQLAlchemy models
  database/                 # Migrations and connectors
  functions/                # Tool functions and MCP client

docs/                       # Source of truth for architecture
tests/                      # Python test suite
scripts/                    # Utility scripts
```

## Environment Setup

**For Jules or any CI agent:**

```bash
# Rust setup
cd capture && cargo build --workspace

# Python setup
pip install -e .
# or
poetry install

# Run full CI
just ci
```

**Pre-requisites (Ubuntu/Linux):**
```bash
# Tesseract for OCR
apt install tesseract-ocr libtesseract-dev

# For audio features (optional in headless)
apt install libasound2-dev
```

## Available Commands

```bash
just check    # Cargo check (fast validation)
just test     # Run all tests (Rust + Python)
just lint     # Clippy + ruff
just ci       # Full CI: check + test + lint
```

## Code Conventions

**Rust (`capture/`):**
- Each crate has clear responsibility
- Use `anyhow` for error handling
- Async runtime: Tokio
- Feature flags for optional dependencies

**Python (`agents/`):**
- Type hints required
- Use `ruff` for linting
- SQLAlchemy for ORM
- Pydantic for configs

## What Works in Headless Linux

| Feature | Headless? | Notes |
|---------|-----------|-------|
| Rust builds | Yes | All crates compile |
| Python agents | Yes | Full functionality |
| Database migrations | Yes | PostgreSQL/SQLite |
| Unit tests | Yes | `just test` |
| OCR (Tesseract) | Yes | Works headless |
| Screen capture | No | Requires display |
| Audio capture | No | Requires audio devices |

## Key Files

- `docs/architecture/overview.md` — System architecture
- `docs/README.md` — Documentation index
- `CLAUDE.md` — Claude Code instructions
- `justfile` — Task runner commands
- `pyproject.toml` — Python project config
- `capture/Cargo.toml` — Rust workspace config

## Memory Agent Types

The `agents/` package implements a MIRIX-derived memory system:

| Agent | Purpose |
|-------|---------|
| Episodic | What happened and when (events, sessions) |
| Semantic | Extracted facts and knowledge |
| Procedural | Workflows and patterns |
| Resource | Files, links, references |
| Knowledge Vault | Curated, verified info |
| Core | Persistent identity/preferences |
| Meta | Memory about memories |
| Reflexion | Self-improvement patterns |

## Testing Strategy

```bash
# Rust tests
cd capture && cargo test --workspace

# Python tests
pytest

# Both
just test
```

## Common Tasks

**Add a new Rust crate:**
1. Create directory under `capture/`
2. Add to `capture/Cargo.toml` workspace members
3. Follow existing crate patterns

**Add a new Python agent:**
1. Create in `agents/agent/`
2. Follow `*_memory_agent.py` pattern
3. Add to `agents/agent/__init__.py`

**Database migrations:**
- PostgreSQL: `agents/database/run_postgresql_migration.py`
- SQLite: `agents/database/run_sqlite_migration.py`

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
