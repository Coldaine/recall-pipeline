---
doc_type: playbook
subsystem: general
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# Agent Playbook

- Read `docs/README.md` and `architecture/overview.md` before coding.
- Default to Python orchestration; Rust agents stay behind flags.
- Do not reintroduce archived/old plans; any new scope requires updating docs here first.
- When routing/agent logic changes, update `domains/routing.md` and log it in `worklog.md` and `changelog.md`.
