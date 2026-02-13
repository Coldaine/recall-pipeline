---
doc_type: architecture
subsystem: routing
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# Routing Domain

## Current Scope (Simplified)

Single server receives all data. No multi-node routing for now.

```
Deployment A ──┐
Deployment B ──┼──▶ Server (Postgres)
Deployment C ──┘
```

## deployment_id

Each capture device has a `deployment_id` configured (e.g., "laptop", "desktop", "work-pc"). All frames are tagged with this ID.

## Future Scope (Not Implemented)

If needed later:
- Multi-sink routing (Postgres, webhook, message queue)
- Active deployment detection (heartbeats)
- Output routing to originating node

For now: everything goes to one Postgres instance.
