---
doc_type: architecture
subsystem: general
version: 1.1.0
status: active
owners: Patrick
last_reviewed: 2026-02-09
issue: 36
---

# Unified Vision: Recall Pipeline

> **Document Type:** Vision capture document. This records conceptual discussions and architectural direction. For concrete implementation phases and code sources, see `overview.md`.

**GitHub Issue**: [#36](https://github.com/Coldaine/recall-pipeline/issues/36)

---

## Core Purpose

**Total digital recall + perfect AI context.** Capture everything across all machines, summarize intelligently at multiple levels, make it searchable and usable as context for LLMs.

---

## 8 Conceptual Questions & Answers

### 1. What problem are you solving for yourself?
> Is this about "I can't remember what I was working on last Tuesday" or "I want an AI that knows my full context" or something else entirely?

**Answer:** Both of those issues, I want to both recall, and give perfect context. The scope of this is large.

---

### 2. What are your "deployments"?
> Are these different machines (laptop, desktop, work PC)? Different contexts (work vs personal)? Or something else?

**Answer:** Different machines, eventually we will aggregate across all of them.

---

### 3. When you imagine using this successfully, what does that interaction look like?
> Are you searching a timeline? Asking an AI questions about your past? Getting automatic summaries? Something passive that just "knows"?

**Answer:** Yes, I want to be able to visualize in dashboards what I was doing at each part of the day, with intelligent summarization, and having LLM review that intelligently sees what I was doing and aggregates that into multiple levels that I can drill down into or up from eg: working on personal projects → working on screenpipe → working on the database layer.

I also want it to capture things like secrets, and redact and store them (securely, eventually we will get into specifics, but I have a FIDO2 key that I want all sensitive information to be encrypted with).

Additionally, I want the MIRIX memory component to explore and research it, to see how useful it is in providing context to my LLMs automatically.

---

### 4. Where should the data live?
> All local on each machine? Synced to a central server you control? Some hybrid? What's your comfort level with your screen history existing on a network?

**Answer:** Ultimately a hierarchy: Local caching/storage gets uploaded, reviewed, etc... or stored wholesale on my remote aggregation server. Key to this vision is the "lazy review" where we are going to do both OCR and LLM vision review of the (de-duplicated) screen content, but certainly not realtime (or perhaps configurable for OCR - for example my laptop probably can't spare the power to do that in realtime, my desktop probably can).

---

### 5. Memory types - necessary or over-engineered?
> The MIRIX system has 6 memory types (episodic, semantic, procedural, resource, knowledge vault, core). Do you actually need distinct memory "types" or is that over-engineered for your use case? Or put differently: what's the difference between "stuff I saw" and "stuff I learned"?

**Answer:** This is for experimentation, and already should exist here, this should be mostly done.

---

### 6. What's your relationship with the captured data over time?
> Is 90-day retention right? Do you want indefinite history? Does old data become less valuable or more valuable?

**Answer:** For testing purposes we're just going to retain, 90 day retention is about right, but we're going to configure this over time.

---

### 7. How much should this system "think" vs just "store"?
> Should it be doing summarization, consolidation, and inference automatically? Or should it mostly be a searchable archive that you query?

**Answer:** Yes, automatically. But we need to flesh this out over time.

---

### 8. Is this purely for you, or do you imagine sharing context with AI assistants?
> If so, how much context should those assistants have access to?

**Answer:** Absolutely, needs to be a catch all storage.

---

## Data Flow Architecture (Server-First)

> **Note:** This diagram shows the simplified server-first architecture. See `overview.md` for implementation phases.

```
DEPLOYMENT (laptop/desktop)              SERVER (Postgres)
┌──────────────────────────┐            ┌─────────────────────────────┐
│                          │            │                             │
│  Screen Capture (Rust)   │            │  Postgres + pgvector        │
│         ↓                │            │    - frames                 │
│  Dedup (phash in memory) │   HTTP     │    - activities             │
│         ↓                │ ────────▶  │    - projects               │
│  POST frame to server    │            │    - embeddings             │
│                          │            │                             │
│  (no local DB - just     │            │  Workers (Python)           │
│   memory buffer + retry) │            │    - OCR (lazy)             │
│                          │            │    - LLM Vision (lazy)      │
└──────────────────────────┘            │    - Summarization (lazy)   │
                                        │                             │
                                        │  Secret Detection           │
                                        │    - Pattern matching       │
                                        │    - Redaction              │
                                        │    - Vault (unencrypted)    │
                                        │                             │
                                        │  Memory Agents (Future)     │
                                        │    - See Phase 5 roadmap    │
                                        │                             │
                                        │  Context API                │
                                        │    (for Claude, etc.)       │
                                        └─────────────────────────────┘
```

**Why server-first?** SQLite write contention killed the original screenpipe local-first approach. Single Postgres instance is simpler and more reliable.

---

## Key Concepts

| Concept | Description |
|---------|-------------|
| **deployment_id** | Which machine captured this (laptop, desktop, etc.) |
| **Lazy review** | OCR/LLM processing is async, batched, configurable per deployment |
| **Hierarchical summaries** | Drill down: Day → Project → Task → Frame |
| **Secret vault** | Detected secrets are redacted from search, stored in a (currently unencrypted) vault |
| **Memory agents** | Automatic LLM context enrichment (Phase 5, experimental) |

---

## What Changes from Current State

| Current | Unified |
|---------|---------|
| Legacy screenpipe schemas removed | Clean recall-* crates with deployment_id |
| user_id / organization_id everywhere | Remove - single user |
| No summarization hierarchy | Add: frame → activity → project → day |
| No secret handling | Add: detection, redaction, secure vault |
| Real-time OCR assumed | Configurable: real-time or lazy per deployment |

---

## Storage Model (Simplified)

```sql
-- Core capture (Rust writes, Python reads)
frames (
  id, deployment_id, captured_at,
  phash, image_ref,
  ocr_text, ocr_status,
  vision_summary, vision_status,
  activity_id  -- links to summarization hierarchy
)

-- Summarization hierarchy
activities (id, deployment_id, start_at, end_at, summary, project_id)
projects (id, name, summary)
day_summaries (id, deployment_id, date, summary)

-- Secret vault (encrypted, separate)
-- Secret vault (unencrypted, separate)
secrets (id, frame_id, raw_value, key_id) -- key_id is for future encryption

-- Memory agents (Phase 5, experimental)
episodic_memory (...)
semantic_memory (...)
-- etc.
```

---

## Related

- Implementation plan: `docs/architecture/overview.md`
- Schema unification: `docs/plans/schema-unification-plan.md`
- GitHub Issue: [#36 - North Star: Conceptual Questions](https://github.com/Coldaine/recall-pipeline/issues/36)
