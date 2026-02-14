# Recall Pipeline

Recall Pipeline is a multi-modal screen capture and analysis system. It captures screen frames, performs deduplication, efficiently stores them in a Postgres database (TimescaleDB + pgvector), and processes them with OCR and Vision ML models to build a semantic memory of user activity.

## Getting Started

### Prerequisites
- Python 3.10+
- Rust 1.75+
- Postgres 16+ (with TimescaleDB and pgvector extensions)

### Documentation
-   **[Secrets Management](docs/secrets_management.md)**: How to decrypt configuration and manage secrets securely using SOPS+age.
-   **[Testing Guide](docs/testing.md)**: How to run Python and Rust tests using the cloud test database.

### Quick Start
1.  **Decrypt Config**:
    ```powershell
    # Requires bws, sops, age installed
    bws secret get AGE_PRIVATE_KEY -o json | ConvertFrom-Json | % { $env:SOPS_AGE_KEY = $_.value }
    sops -d .env.enc > .env
    ```

2.  **Run Python Tests**:
    ```powershell
    .\.venv\Scripts\activate
    pytest
    ```

3.  **Run Rust Tests**:
    ```powershell
    cargo test --workspace
    ```
