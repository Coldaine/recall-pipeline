---
doc_type: index
subsystem: general
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# Documentation Home

This folder is the single source of truth for architecture and delivery. Treat any other Markdown in the repo as legacy/auxiliary; do not source requirements from it.

- `architecture/` — high-level overview, guiding principles, and [screenpipe crate audit](architecture/screenpipe-crate-audit.md).
- `domains/` — per-area expectations (capture, storage, orchestration, routing).
- `dev/` — build, CI, testing, observability practices.
- `project-management/` — changelog, worklog, and agent playbook for contributions.

Default stance: Rust for capture/storage/LLM workers (hot path), Python for orchestration/agents. Rust-native agents are optional/experimental and must remain behind flags.
