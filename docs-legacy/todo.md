---
last_edited: 2026-02-13
editor: Claude Code (Claude Opus 4.5)
user: Coldaine
status: active
version: 1.0.0
subsystem: project-management
tags: [tasks, backlog, todo]
doc_type: plan
---

# Project Tasks (Single Source of Truth)

## Active (Phase 2: Lazy Processing)

- [ ] **Implement OCR Worker** (Rust)
    - Poll `frames` where `ocr_status = 0`
    - Run Tesseract/OCR engine
    - Update `ocr_text` and set `ocr_status = 1`
    - Handle errors (`ocr_status = 2`)
- [ ] **Implement Vision Worker** (Rust)
    - Poll `frames` where `vision_status = 0`
    - Call Vision LLM (Candle/External)
    - Store summary
- [ ] **Implement Summarization**
    - Group frames into 15-min buckets
    - Generate activity summaries

## Backlog

- [ ] **Phase 3: Hierarchical Summaries**
    - Create `activities`, `projects`, `day_summaries` tables (migration)
    - Implement aggregation logic
- [ ] **Phase 4: Secrets Management**
    - Integrate FIDO2 decryption
    - Implement regex redaction in capture loop
- [ ] **Phase 5: MIRIX Integration**
    - Port MIRIX memory types to Rust

## Completed (Phase 1)

- [x] **Raw Capture** (Rust)
    - Screen capture
    - Phash deduplication
    - Postgres storage
- [x] **Database Schema** (Baseline)
    - `frames` table created
    - `pgvector` enabled
