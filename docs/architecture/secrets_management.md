---
last_edited: 2026-02-17
editor: Antigravity (Claude-3.5-Sonnet)
user: Coldaine
status: ready
version: 1.0.0
subsystem: architecture
tags: [secrets, security, sops, age]
doc_type: architecture
---

# Secrets Management


We use **SOPS** (Secrets OPerationS) with **age** encryption to manage sensitive configuration values in this repository. This allows us to commit encrypted `.env` files to git while keeping the keys secure and portable.

## Prerequisites
- **SOPS**: Encrypts/decrypts files. (`scoop install sops` on Windows)
- **age**: Generates encryption keys. (`scoop install age` on Windows)
- **Bitwarden Secrets Manager (`bws`)**: Stores the master private key.

## Setup for New Clones
1.  **Install tools**:
    ```powershell
    scoop install sops age bws
    ```
2.  **Authenticate `bws`**:
    Ensure `BWS_ACCESS_TOKEN` is set, or run `bws login`.
3.  **Fetch the Private Key**:
    Retrieve the `age` private key from Bitwarden and set it in your environment.
    ```powershell
    $key = bws secret get AGE_PRIVATE_KEY --output json | ConvertFrom-Json
    $env:SOPS_AGE_KEY = $key.value
    ```
    *(Tip: Add this to your shell profile for persistence)*
4.  **Decrypt Secrets**:
    Generate the local `.env` file from the encrypted source.
    ```powershell
    sops --decrypt .env.enc > .env
    ```

## Adding/Updating Secrets
1.  **Edit Encrypted File**:
    This opens your default editor, decrypts the file in memory, and re-encrypts on save.
    ```powershell
    sops .env.enc
    ```
2.  **Commit Changes**:
    Since `.env.enc` is encrypted, it is safe to commit and push to git.
    ```bash
    git add .env.enc
    git commit -m "Update secrets"
    ```

## CI/CD (GitHub Actions)
The `SOPS_AGE_KEY` is stored as a GitHub Action secret. The workflow uses it to decrypt `.env.enc` before running tests or deployments.
