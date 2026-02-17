# System Architecture Overview

> **Context**: High-level view of the Recall Pipeline system.

## The Loop
1.  **Capture**: `recall-capture` (Rust) runs on the edge device. It captures screen frames, diffs them, and sends relevant ones to the database.
2.  **Ingest**: `recall-server` (Rust) receives frames and writes them to Postgres.
3.  **Process**: `recall-agents` (Python) wake up on new data, run OCR/Vision analysis, and generate summaries.
4.  **Recall**: User queries the system via the `recall-ui` or API to find past content.

## Domain Boundaries

### Capture Domain
- **Responsibility**: Get pixels from screen to DB.
- **Components**: `xcap` (screen recording), `image-compare` (diffing), `postgres` (client).
- **Constraints**: maximize performance, minimize resource usage. Stop for nothing.

### Storage Domain
- **Responsibility**: Source of Truth.
- **Components**: PostgreSQL + `pgvector`.
- **Schema**: See `storage/st-overview.md`.

### Orchestration Domain
- **Responsibility**: Make sense of the data.
- **Components**: Python agents, OCR workers, LLM summarizers.
- **Pattern**: Lazy, async processing. Ingest is fast; understanding is slow.
