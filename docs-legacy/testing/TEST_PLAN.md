# Rust Testing Plan & Status

## Overview

This document outlines the current state of testing for the Rust crates and future improvements.

## Philosophy

Follows `docs/TESTING_PHILOSOPHY.md`:
- **Real Dependencies:** Use real Postgres, real filesystem.
- **Integration over Unit:** Test workflows, not just functions.
- **Exceptions:** Unit tests for pure algorithms (e.g., perceptual hash).

## Current Status (Feb 2026)

### 1. recall-store
- **Grade: A**
- **Coverage:** 
    - Full lifecycle integration test (`store_integration.rs`).
    - Filesystem edge cases (`image_storage_edge.rs`).
- **Notes:** Excellent adherence to philosophy.

### 2. recall-db
- **Grade: B**
- **Coverage:** Connectivity smoke test.
- **Notes:** Sufficient for current complexity.

### 3. recall-capture
- **Grade: B** (Improved from B-)
- **Coverage:** 
    - Algorithm unit tests (dedup).
    - Pipeline integration test (`pipeline_integration.rs`).
- **Notes:** Now includes a full pipeline test that verifies the flow from capture to storage using real channels and a real (test) DB.

## TODOs

- [ ] Add error handling tests for `recall-capture` pipeline (e.g., storage failure).
- [ ] Add more complex deduplication scenarios to `pipeline_integration.rs`.
- [ ] implementation of OCR integration tests when OCR component is ready.
