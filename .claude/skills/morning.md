---
name: morning
description: Morning standup - read context, pick next task, implement
user-invocable: true
---

# Morning Standup

Your mandatory pre-task workflow. Run this every session start.

## Step 1: Read Context

Read these in order:
1. **docs/Northstar.md** — Immutable principles (no SQLite, pure Rust, lazy processing, deployment_id everywhere)
2. **docs/index.md** — Documentation index (architecture, domains, dev philosophy)
3. **todo.md** — Task backlog (what's pending, in progress, completed)

## Step 2: Understand Architecture

From the docs, confirm you understand:
- **Capture**: Rust + xcap for screen recording (in `capture/` workspace)
- **Storage**: PostgreSQL only + pgvector (no SQLite, no local DB)
- **Orchestration**: Python agents for OCR/vision/summarization
- **Constraints**: Single user (no multi-tenant), deployment_id on every table

## Step 3: List Incomplete Tasks

Review `todo.md` and list all tasks with status `pending` or `in_progress`.

Ask me: "Here are the tasks I see. Which should I work on next?"

## Step 4: Get Approval & Implement

Once I pick a task:
1. Enter Plan Mode (Shift+Tab or --permission-mode plan)
2. Read the relevant code files
3. Propose a plan
4. Wait for approval
5. Implement carefully (don't over-engineer)
6. Run tests & clippy
7. Commit with clear message

## Rules
- ✅ Read code before changing it
- ✅ Ask questions if unclear
- ✅ Run tests after changes
- ❌ Don't guess about intent
- ❌ Don't add features beyond scope
- ❌ Don't delete without explicit confirmation
