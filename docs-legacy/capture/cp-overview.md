---
doc_type: architecture
subsystem: capture
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2026-02-09
---
# Capture Domain

## Scope

Screen capture only. Audio is out of scope.

## Implementation

- Rust (`capture/recall-capture/`) handles screen capture and phash dedup via xcap 0.8. OCR runs server-side (Phase 2).
- Target platforms: Windows/macOS/Linux
- Writes frames via HTTP POST to server (no local database)

## Required Fields Per Frame

| Field | Source |
|-------|--------|
| `deployment_id` | Config (identifies which machine) |
| `captured_at` | Timestamp of capture |
| `phash` | Computed in Rust before POST |
| `window_title` | OS API |
| `app_name` | OS API |
| `image_ref` | Path/URL after image stored |

## Reliability

- Buffer in memory if network unavailable
- Retry with backoff on POST failure
- Don't block capture loop on network issues

## What's NOT in capture domain

- OCR processing (lazy, on server)
- LLM vision summarization (lazy, on server)
- Embeddings (lazy, on server)
- Audio (out of scope)
