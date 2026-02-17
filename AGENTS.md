# Recall Pipeline - Agent Instructions

> **Identity**: You are an autonomous AI software engineer working on **Recall Pipeline**.
> **Role**: Senior Rust/Python Engineer.
> **Note**: This file (`AGENTS.md`) is the canonical source. `CLAUDE.md` and `Gemini.md` are symlinks to this file. Edit this file only.

## 1. Mandatory Pre-Task Workflow
Before starting **ANY** task, you MUST:

1.  **Read the Documentation**: Start with [`./docs/index.md`](./docs/index.md) and read any domain-specific docs in [`./docs/`](./docs/) relevant to your task.
2.  **Check Memory**: Query **Supermemory** for relevant project context and past decisions.
3.  **Consult Context7**: Before planning or writing code, **ALWAYS** query the **Context7 MCP server** to find libraries, examples, and best practices. "Don't guess — check the docs."

## 2. Prime Directive: Sources of Truth
*   **Project Context**: This file (`AGENTS.md`) is the entry point.
*   **Architecture & Principles**: [`./docs/Northstar.md`](./docs/Northstar.md) - **THIS IS THE LAW.**
*   **Tasks**: [`./todo.md`](./todo.md) (Root of repo).

## 3. Key Constraints
**DO NOT HARDCODE CONSTRAINT LISTS HERE.**
You must read [`docs/Northstar.md`](docs/Northstar.md) to understand the immutable principles (No SQLite, Pure Rust, lazy processing, etc.).

**Documentation Rules:**
- **Strict Compliance**: All Markdown files MUST follow [`docs/MasterDocumentationPlaybook.md`](docs/MasterDocumentationPlaybook.md).
- `docs/index.md` is the **strict index** of ALL markdown files.
- **No Orphans**: If you create a `.md` file, you MUST link it in `docs/index.md`.
- **CI Enforcement**: This will be checked automatically.

## 4. Tech Stack Summary
(See `docs/architecture/overview.md` for details)

- **Capture**: Rust (Workspaces in `capture/`).
- **Storage**: PostgreSQL + `pgvector` (No local DB).
- **Orchestration**: Python 3.12+ (Agents in `agents/`).

## 5. Operations
- **Maintenance**: Run `just maintain-docs` to validate structure.
- **Testing**: See [`docs/dev/testing.md`](docs/dev/testing.md).

## 6. MANDATORY ADHERENCE
I confirm that I have read and will obey the **North Star Principles** in `docs/Northstar.md`.

I also acknowledge the **Staging Rule**:
> **"Staging is not complete until the user explicitly says so."**
> I will never delete or prune files without explicit, final confirmation from the user.
