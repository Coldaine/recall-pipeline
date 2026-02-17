# Justfile for Recall Pipeline Documentation

default:
    @just --list

# Validate documentation structure and naming
validate-docs:
    @echo "Running documentation validation..."
    # TODO: Add python script call here once created
    @echo "Checking for required root files..."
    @if not exist "MasterDocumentationPlaybook.md" (echo "MISSING: MasterDocumentationPlaybook.md" && exit 1)
    @if not exist "Gemini.md" (echo "MISSING: Gemini.md" && exit 1)
    @if not exist "Agent.md" (echo "MISSING: Agent.md" && exit 1)
    @echo "Validation Passed."

# Sync tasks (placeholder for future automation)
sync-tasks:
    @echo "Syncing tasks from todo.md..."

# Maintain docs (full check)
maintain-docs: validate-docs
    @echo "Documentation maintenance complete."

# -----------------------------------------------------------------------------
# Testing & Linting
# -----------------------------------------------------------------------------

# Run all tests (Rust unit + Python unit)
test: test-rust test-python

# Rust workspace tests (unit + db integration if DATABASE_URL set)
test-rust:
    cd capture && cargo test --workspace

# Python tests via pytest
test-python:
    python -m pytest tests/ -v

# Hardware tests (require a display + monitors)
# Note: This captures actual screens. Do not run in headless CI without a virtual display.
test-hw:
    cd capture && cargo test --package recall-capture --test pipeline_integration -- --nocapture

# Lint all
lint: lint-rust lint-python

# Rust clippy
lint-rust:
    cd capture && cargo clippy --workspace -- -D warnings

# Python ruff
lint-python:
    python -m ruff check agents/ tests/
