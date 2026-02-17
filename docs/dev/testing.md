---
last_edited: 2026-02-17
editor: Claude Code (Claude Opus 4.5)
user: Coldaine
status: ready
version: 1.0.0
subsystem: dev
tags: [testing, rust, python, ci, guide]
doc_type: guide
---

# Testing Strategy

This document outlines the testing strategy for the Recall Pipeline monorepo. It covers Rust (Capture/Storage), Python (Agents/OCR), and cross-component integration.

## 1. Test Categories

We categorize tests into three levels:

1.  **Unit Tests** (Fast, No External Deps)
    - **Rust**: Inline `#[test]` functions within `src/`.
    - **Python**: `pytest` tests in `tests/` mocking external calls.
    - **Goal**: Verify logic, parsing, configuration, and data models.
    - **Run**: Always (Local & CI).

2.  **Database Integration Tests** (Requires Postgres)
    - **Rust**: Integration tests in `capture/*/tests/*.rs`.
    - **Python**: `pytest` tests marked with `integration`.
    - **Mechanism**: Gated by `DATABASE_URL` environment variable.
    - **Run**:
        - **Local**: Run if you have a local DB (`just test` auto-detects).
        - **CI**: Run via Service Container + GitHub Secrets.

3.  **Hardware Isolation Tests** (Requires Display)
    - **Rust**: `capture/recall-capture/tests/pipeline_integration.rs`.
    - **Goal**: Verify screen capture on actual hardware.
    - **Run**:
        - **Local**: Explicitly via `just test-hw`.
        - **CI**: Skipped (headless runners lack displays).

## 2. Running Tests

We use `just` as the unified test runner.

| Command | Description | Prerequisites |
|---------|-------------|---------------|
| `just test` | Run all Unit + DB Integration tests | `DATABASE_URL` (optional) |
| `just test-rust` | Run only Rust workspace tests | Rust toolchain, `DATABASE_URL` (optional) |
| `just test-python` | Run only Python `pytest` suite | Python venv |
| `just test-hw` | Run Hardware Capture tests | **Physical Display** |

### Environment Setup

#### Database Tests
To run database integration tests locally:
1. Start Postgres (e.g., via Docker):
   ```bash
   docker run --name recall-db -e POSTGRES_PASSWORD=recall -p 5432:5432 -d postgres:16
   ```
2. Set the environment variable:
   ```powershell
   $env:DATABASE_URL="postgresql://recall:recall@localhost:5432/recall"
   ```
3. Run tests:
   ```bash
   just test-rust
   ```
   *Tests will automatically skips if `DATABASE_URL` is unset.*

## 3. Structure & Conventions

### Rust
- **Unit Tests**: Place in the same file as the code module (`mod tests`).
- **Integration Tests**: Place in `tests/` directory at the crate root.
- **Data**: Use `recall-db` migrations to set up schema; wrap tests in transactions that roll back OR use unique IDs/deployments to avoid collisions.

### Python
- **Location**: `tests/` at the repo root.
- **Runner**: `pytest`.
- **Fixtures**: Defined in `tests/conftest.py`.
- **Gating**: Use the `db_url` fixture to skip tests requiring a database.

## 4. CI Strategy

Our GitHub Actions workflow handles tests as follows:

1.  **Lint & Format**: Runs `cargo fmt`, `clippy`, `ruff` first.
2.  **Unit Tests**: Runs `cargo test --lib` and `pytest` (excluding integration).
3.  **Integration Tests**:
    - **Services**: Spins up a Postgres service container.
    - **Secrets**: Injects `DATABASE_URL` from repository secrets.
    - **Execution**: Runs `cargo test --test '*'` to execute integration suites.
