---
last_edited: 2026-02-17
editor: Antigravity (Claude-3.5-Sonnet)
user: Coldaine
status: draft
version: 1.0.0
subsystem: capture
tags: [capture, overview, technical]
doc_type: index
domain_code: cp
---

# Capture Domain (cp) Overview

The Capture domain is responsible for acquiring data from various sources (screen, audio, window metadata) using Rust-based infrastructure.

## Components
- **recall-capture**: Core Rust crate for screen and audio capture.
- **Deduplication**: Frame-level diffing to minimize storage.

## Linked Documents
- [Capture ADR (ADR-003)](../../architecture/adr-003.md)
