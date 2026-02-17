---
doc_type: plan
subsystem: general
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# Project Consolidation (November 2025)

This document records what was removed and renamed during the November 2025 consolidation. This is a historical record for reference.

## What Got Removed

| Removed | Why |
|---------|-----|
| `recall_pipeline/` | Legacy Python capture code, replaced by Rust capture engine |
| `screenpipe/screenpipe-app-tauri/` | Desktop GUI application we don't need |
| `screenpipe/pipes/` | Example applications and plugins |
| `screenpipe/screenpipe-js/` | JavaScript SDK |
| `screenpipe/content/` | Marketing images and documentation assets |

## What Got Renamed

| From | To | Why |
|------|-----|-----|
| `screenpipe/` | `capture/` | Cleaner naming, matches architecture domain terminology |

## Context

The project was built by extracting from two open-source projects:

1. **MIRIX** (Python multi-agent memory system) → Integrated into `agents/`
2. **screenpipe** (Rust screen capture engine) → Extracted into `capture/`

The consolidation removed:
- screenpipe's desktop app (Tauri) — we're building a server-first system
- screenpipe's plugin/pipe system — not needed for our use case
- The legacy Python capture code — superseded by Rust implementation
- Marketing and documentation assets from screenpipe

## Related

- Architecture: `docs/architecture/overview.md`
- Code inventory: `docs/architecture/code-inventory.md`
- Implementation roadmap: `docs/plans/implementation-roadmap.md`
