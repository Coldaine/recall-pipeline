# Agent / Project Context

## Project: Recall Pipeline

**Goal**: Create a personal "Google" for my life history. Record everything, summarize it, and make it queryable.

## Architecture: The Hybrid Model

We use the right tool for the job:

### 1. The Edge (Rust) 🦀
- **Responsibility**: Capture screens, calculate hashes, handle device I/O.
- **Why Rust?**: Performance, safety, low memory footprint. We cannot block the user's machine.
- **Code**: `capture/` directory.

### 2. The Center (Postgres) 🐘
- **Responsibility**: Single Source of Truth.
- **Why Postgres?**: Concurrency, `pgvector`, TimescaleDB. SQLite failed us.
- **Location**: Docker container / Dedicated Server.

### 3. The Brain (Python + Rust) 🐍
- **Responsibility**: "Lazy" asynchronous processing.
    - OCR (Tesseract/EasyOCR)
    - Vision (VLM summarization)
    - Embeddings
    - Knowledge Graph (MIRIX)
- **Why Hybrid?**: Python has the best AI ecosystem. Rust has the best performance for heavy compute.
- **Code**: `agents/` directory.

## Current State
- **Phase**: 2 (Lazy Processing).
- **Focus**: Building the async workers to process the raw frames captured by Rust.
