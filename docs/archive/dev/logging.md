---
doc_type: standard
subsystem: general
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# Logging and Tracing

- Use structured logging with deployment and agent identifiers.
- Emit metrics for success/error counts, latency, token usage, and budget rejections.
- Ensure correlation IDs: `agent_run_id`, `deployment_id` on all spans/events.
