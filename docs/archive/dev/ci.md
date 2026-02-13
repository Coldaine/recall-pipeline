---
doc_type: playbook
subsystem: general
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# CI Plan

## Principle

CI expands with each phase. Only test what's implemented.

## Phase 1: Raw Capture

```yaml
jobs:
  rust-capture:
    - cargo build -p screenpipe-vision
    - cargo test -p screenpipe-vision

  python-ingest:
    - pytest agents/server/test_frame_ingest.py

  integration:
    - Start Postgres
    - Run capture → POST → verify row exists
```

## Phase 2: Lazy Processing

```yaml
jobs:
  # ... Phase 1 jobs ...

  python-workers:
    - pytest agents/processors/

  integration-ocr:
    - Insert frame → OCR worker runs → verify ocr_text

  integration-vision:
    - Insert frame → Vision worker runs → verify vision_summary
```

## Phase 3: Hierarchical Summaries

```yaml
jobs:
  # ... Phase 1-2 jobs ...

  python-summarization:
    - pytest agents/services/summarization/

  integration-summaries:
    - Frames exist → summarizer runs → activities created
```

## Phase 4: Memory Agents

```yaml
jobs:
  # ... Phase 1-3 jobs ...

  python-memory:
    - pytest agents/agent/
    - pytest agents/services/*_memory_manager.py
```

## Not Testing (Removed)

- Multi-node routing simulation
- Backpressure/load testing
- Audio capture
