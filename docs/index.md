---
last_edited: 2026-02-17
editor: Antigravity (Claude-3.5-Sonnet)
user: Coldaine
status: ready
version: 1.1.0
subsystem: monorepo
tags: [index, documentation, overview]
doc_type: index
---

# Documentation Index


> **Auto-Generated Candidate**: This file is the single entry point for all documentation in the repository.
> **CI Enforcement**: Every Markdown file in `docs/` MUST be linked here. No orphans allowed.

## Core
- [Northstar Principles](Northstar.md)
- [Master Documentation Playbook](MasterDocumentationPlaybook.md)
- [Task Backlog](../todo.md)

## Architecture
- [System Overview](architecture/overview.md)
- [Roadmap](architecture/roadmap.md)
- [ADR-001: PostgreSQL Only](architecture/adr-001.md)
- [ADR-002: Pure Rust Stack](architecture/adr-002.md)
- [ADR-003: Capture Rewrite from Screenpipe](architecture/adr-003.md)
- [ADR-004: Hooks-as-Primitives Development Harness](architecture/adr-004.md)
- [Secrets Management](architecture/secrets_management.md)

## Domains
### Capture
- [Overview & Specs](domains/capture/cp-overview.md)

### Storage
- [Database Schema](domains/storage/st-overview.md)


## Developer
- [Testing Philosophy](dev/testing.md)
- [Logging Philosophy](dev/logging.md)
- [Claude Code Hooks Operational Guide](dev/hooks-harness.md)

## Archive
- [Original Screenpipe Audit](archive/screenpipe-crate-audit.md)

## Configuration Index
> **Context**: Root-level configuration files and hidden folders.

- [`.gitignore`](../.gitignore): Defines what git ignores (target/, .env, etc.)
- [`.vscode/`](../.vscode/): Editor settings (extensions, tasks, launch config).
- [`justfile`](../justfile): Command runner (build, test, deploy).
- [`Cargo.toml`](../capture/Cargo.toml): Rust workspace definition.
- [`pyproject.toml`](../pyproject.toml): Python dependencies and tools.
- [`AGENTS.md`](../AGENTS.md): Agent Instructions (System Prompt).

