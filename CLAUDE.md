# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Single-user, multi-deployment capture and memory system for total digital recall + perfect AI context. Capture everything across all machines, summarize hierarchically (Frame → Activity → Project → Day), make it searchable and usable as context for LLMs.

**Stack split**: Rust handles capture/storage/LLM workers (hot path); Python orchestrates memory agents.

## Documentation

@docs/README.md
@docs/architecture/overview.md
@docs/project-management/worklog.md

---

## Development Commands

### Python (agents/)

```bash
uv sync                      # Install dependencies
uv sync --extra dev          # Install with dev tools
uv run pytest tests/ -v      # Run all tests
uv run pytest tests/test_foo.py::test_bar -v  # Run single test
uv run ruff check agents/    # Lint
uv run ruff check --fix agents/  # Auto-fix lint issues
uv run mypy agents/ --ignore-missing-imports  # Type check
```

### Rust (capture/)

```bash
cd capture
cargo check --workspace      # Fast check (no build)
cargo fmt --all -- --check   # Check formatting
cargo fmt --all              # Auto-format
cargo clippy --workspace -- -D warnings  # Lint
cargo build --workspace      # Full build (requires native deps)
cargo test --workspace       # Run tests
```

**Crates**: `recall-capture` (xcap 0.8 screen capture + dedup), `recall-db` (Postgres/sqlx), `recall-store` (storage traits + adapters). Binary: `recall`.

---

## Key Directories

| Path | Purpose |
|------|---------|
| `capture/` | Rust workspace (recall-capture, recall-db, recall-store) |
| `agents/` | Python memory agents (MIRIX-derived, 6 memory types) |
| `docs/` | Single source of truth for architecture |
| `scripts/` | Utility scripts (DB setup, systemd, etc.) |
| `.claude/commands/` | Custom slash commands |
| `docs/session-logs/` | Session work logs |

---

## Current State & Known Issues

**Rust build status**: Clean crates (recall-capture, recall-db, recall-store) built from scratch. No legacy screenpipe dependencies.

**Next priorities**: See `docs/project-management/worklog.md`

---

## Session Logging

Use `/session-log` before ending a session to document work.

Logs: `docs/session-logs/YYYY-MM-DD-summary.md`

Include: accomplishments, decisions with rationale, blockers, next steps.

---

## Working with Jules (Google's Coding Agent)

[Jules](https://jules.google) is Google's async coding agent powered by Gemini 2.5 Pro. It clones repos into Ubuntu VMs with Node.js, Python, Go, Rust, Java, Docker, GCC, Clang, CMake preinstalled.

**Environment Setup**: Jules reads `AGENTS.md` or `README.md` for setup hints, or uses custom setup scripts configured in repo settings. With a proper justfile/makefile, it can install native dependencies via apt. Environment Snapshots cache complex setups for faster execution.

**Good for**: Well-scoped tasks - bug fixes, test writing, dependency updates, small features, docs, refactoring.
**Limitations**: GitHub-only, struggles with architectural overhauls, needs human review (like a junior dev).

**Workflow**:
1. Create well-scoped GitHub issue with clear acceptance criteria
2. `gh issue edit <number> --add-label "jules"`
3. Review the PR when Jules submits it

**Task limits**: 100/day with 15 concurrent (Pro plan).
