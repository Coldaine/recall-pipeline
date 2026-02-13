# Gemini / Agent Instructions

> **Identity**: You are an autonomous AI software engineer working on **Recall Pipeline**.

## 1. Prime Directive: The Playbook
- **Location**: [`./MasterDocumentationPlaybook.md`](./MasterDocumentationPlaybook.md)
- **Rule**: YOU MUST READ THIS FILE before creating or editing documentation.
- **Enforcement**: Strict adherence to folder structure, headers, and retention policies.

## 2. The Vision (North Star)
- **Location**: [`./docs/Northstar.md`](./docs/Northstar.md)
- **Core Goal**: "Total digital recall + perfect AI context."
- **Architecture**:
    1.  **Capture (Rust)**: High-performance, low-overhead capture on edge devices (Laptop, Desktop).
    2.  **Storage (Postgres)**: Centralized, single source of truth. No local SQLite.
    3.  **Intelligence (Python/Rust)**: Lazy, async agents processing frames from Postgres.

## 3. Operations
- **Tasks**: Check [`./docs/todo.md`](./docs/todo.md) for the active backlog.
- **Maintenance**: Run `just maintain-docs` to validate structure (see [`./Justfile`](./Justfile)).

## 4. Coding Standards
- **Rust**: `cargo fmt`, `cargo clippy`.
- **Python**: `ruff`, `mypy`.
- **Docs**: Markdown standards per Playbook.
