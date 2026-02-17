# Logging Philosophy

> **Context**: How we log in Recall Pipeline.

## Principles
1.  **Structured logging**: Use `tracing` crate (Rust) and `structlog` (Python).
2.  ** Levels**:
    - `ERROR`: Wake up operator.
    - `WARN`: Actionable but not critical.
    - `INFO`: Key lifecycle events (startup, shutdown, summary stats).
    - `DEBUG`: meaningful state changes.
    - `TRACE`: detailed loop info (spammy).
3.  **No `println!`**: All output must go through the logging system.
