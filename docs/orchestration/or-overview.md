---
doc_type: architecture
subsystem: orchestration
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# Orchestration Domain

## Scope

Python workers that process captured frames on the server.

## Implementation

- Python (`agents/`) handles all lazy processing
- Runs on server, not on capture devices
- Polls Postgres for pending work

## Workers (Phase 2+)

| Worker | Input | Output |
|--------|-------|--------|
| OCR Worker | Frames where `ocr_status = 0` | `ocr_text`, update `ocr_status = 1` |
| Vision Worker | Frames where `vision_status = 0` | `vision_summary`, update `vision_status = 1` |
| Embedding Worker | Frames where `embedding_status = 0` | `embedding` vector, update `embedding_status = 1` |

## LLM Providers (from MIRIX)

Production-ready clients in `agents/llm_api/`:
- OpenAI ✅
- Anthropic ✅
- Azure OpenAI ✅
- Google AI ✅
- Cohere ❌ (stub)
- Mistral ❌ (partial)

## Error Handling

- Workers should handle LLM rate limits with backoff
- Set status to `2` (error) on persistent failures
- Log errors for debugging

## What's NOT in orchestration

- Real-time processing (all lazy)
- Capture (that's Rust)
- Multi-node routing (removed — single server)
