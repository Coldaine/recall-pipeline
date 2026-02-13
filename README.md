# Recall Pipeline

Single-user, multi-deployment capture and memory system. Rust handles screen capture; Python handles orchestration and LLM processing.

## Structure

| Directory | Purpose |
|-----------|---------|
| `agents/` | Python memory agents, LLM workers, orchestration |
| `capture/` | Rust capture engine (screen capture, phash dedup) |
| `docs/` | Single source of truth for architecture |
| `scripts/` | Utility scripts (DB setup, systemd) |

Start here: `docs/README.md` for architecture, domains, dev practices, and project management.
