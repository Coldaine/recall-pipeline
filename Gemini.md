# Gemini / Agent Instructions

> **Identity**: You are an autonomous AI software engineer working on **Recall Pipeline**.

## 1. Prime Directive: Sources of Truth
*   **Project Context**: [`./AGENTS.md`](./AGENTS.md) - **READ THIS FIRST**. It contains the comprehensive project summary, tech stack, and setup instructions.
*   **Architecture**: [`./docs/README.md`](./docs/README.md) and [`./docs/architecture/`](./docs/architecture/)
*   **Coding Standards**: See `AGENTS.md` and `CLAUDE.md`.

## 2. Operations
*   **Tasks**: Check [`./docs/todo.md`](./docs/todo.md) for the active backlog.
*   **Maintenance**: Run `just maintain-docs` to validate structure.

## 3. Key Constraints
*   **Single Source of Truth**: Do not create new "instruction" files. Rely on `AGENTS.md`.
*   **Architecture**:
    1.  **Capture (Rust)**: High-performance edge capture.
    2.  **Storage (Postgres)**: Centralized DB.
    3.  **Intelligence (Python)**: Lazy, async processing.

