# Testing Guide

This project uses a cloud-based **Neon Postgres** database for integration testing. This ensures tests run in a consistent environment that closely mirrors production.

## Prerequisites
1.  **Database URL**: You need a valid connection string to the test database.
    -   **CI/CD**: This is automatically provided via the `TEST_DATABASE_URL` secret.
    -   **Local**: You can use the shared Neon test database or your local Postgres instance.

2.  **Secrets**: Ensure you have decrypted the `.env` file (see `docs/secrets_management.md`).

## Running Tests
### Python (Integration & E2E)
```bash
# Ensure venv is active and dependencies installed
.\.venv\Scripts\activate
pip install -r requirements.txt

# Run all tests
pytest tests/integration tests/e2e
```

### Rust (Unit & Integration)
The Rust workspace includes three crates: `recall-capture`, `recall-db`, and `recall-store`.

```bash
# Run all tests in the workspace
cargo test --workspace

# Run only logic unit tests (fast, no DB interaction)
cargo test --workspace --lib

# Run integration tests (requires valid DATABASE_URL)
cargo test --workspace --test '*'
```
