---
last_edited: 2026-02-17
editor: Antigravity (Claude-3.5-Sonnet)
user: Coldaine
status: draft
version: 1.0.0
subsystem: storage
tags: [storage, database, postgres, schema]
doc_type: index
domain_code: st
---

# Storage Domain (st) Overview

The Storage domain handles persistence of captured frames, metadata, and embeddings using a PostgreSQL-only architecture.

## Components
- **PostgreSQL**: Primary data store.
- **pgvector**: For vector similarity search.
- **TimescaleDB**: For time-series optimization.

## Linked Documents
- [PostgreSQL Only ADR (ADR-001)](../../architecture/adr-001.md)
