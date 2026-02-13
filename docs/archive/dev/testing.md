---
doc_type: standard
subsystem: general
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# Testing Expectations

## Unit Tests

| Phase | What to Test |
|-------|--------------|
| 1 | Phash computation, frame POST serialization |
| 2 | OCR worker, vision worker, LLM client mocking |
| 3 | Summarization prompts, activity clustering |
| 4 | Memory agent logic, PII detection patterns |

## Integration Tests

| Phase | What to Test |
|-------|--------------|
| 1 | Capture → POST → Postgres row exists |
| 2 | Frame inserted → worker processes → status updated |
| 3 | Frames → summarizer → activities/projects created |
| 4 | Summaries → memory agents → memory tables populated |

## Not Testing (Removed)

- Multi-node routing/backpressure under load
- Audio capture/transcription
- Rust agents (stubs, not production)
