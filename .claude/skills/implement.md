---
name: implement
description: Master workflow - figure out what to do, plan, implement, verify, iterate
user-invocable: true
---

# /implement - Master Development Workflow

Your complete development automation pipeline. One command orchestrates everything.

## How It Works

```
figure-out-what-to-do → plan → implement → test → verify → report
```

Each stage spawns sub-agents in parallel where possible, with you maintaining control.

---

## Stage 1: Figure Out What to Do

### Sub-Agent: Context Briefing (Haiku)

I spawn a haiku agent to brief you on:
- What's in `todo.md` (pending tasks)
- Recent ADR decisions (`docs/architecture/adr-*.md`)
- Architecture principles (`docs/Northstar.md`)
- Any blocking issues from GitHub

**The agent returns:**
```
📋 PENDING TASKS:
  • Task 1 (depends on: Task 2)
  • Task 2 (blocked by: Issue #3)
  • Task 3 (priority: high)

🏗️  ARCHITECTURE CONSTRAINTS:
  • deployment_id on every table
  • No SQLite, pure Postgres
  • Integration tests only (no mocks)

⚠️  BLOCKERS:
  • Issue #15: Schema migration needed
  • PR #42: Waiting for review
```

### Your Decision

You choose: "I want to work on Task 2" (or ask for more context).

---

## Stage 2: Plan

### Enter Plan Mode

I switch to Plan Mode (Shift+Tab) automatically, then:
1. Read the relevant code files
2. Check what research applies (Supermemory + Context7 MCP)
3. Generate a detailed plan
4. Present it to you

**You review the plan and approve/reject/modify.**

---

## Stage 3: Implement (Parallel Sub-Agents)

### Sub-Agent: Code Writer (Sonnet)

Implements the change:
- Reads ALL relevant files first
- Never over-engineers (only requested changes)
- Follows Northstar constraints
- Adds tests if applicable

### Sub-Agent: Documentation Writer (Haiku)

Updates docs in parallel:
- Updates `last_edited` frontmatter
- Updates `docs/index.md` links if new files created
- Adds architecture notes if relevant

**Both run in parallel** → faster execution.

---

## Stage 4: Test & Debug

### Build/Lint Check

```bash
cargo check --workspace
cargo clippy -- -D warnings
cargo test --lib
```

If tests fail:
- **Sub-Agent: Debugger (Haiku)** analyzes errors
- Suggests fixes
- You confirm, then iteratively re-run

### TDD Mode (if applicable)

If this is a new feature:
- **Sub-Agent: Test Writer (Haiku)** writes integration tests first
- You review tests (do they test the right behavior?)
- Code Writer implements to pass tests
- Iterative: test → code → test

**Avoid mocks** unless absolutely unavoidable (external APIs, rate limits).

---

## Stage 5: Quick Pass (Your Review)

You review:
- Code quality
- Compliance with Northstar
- Test coverage
- Documentation updates

You can:
- Approve ("ship it")
- Request changes ("fix X")
- Ask questions ("why Y?")

---

## Stage 6: Verify Compliance (SessionEnd Hook)

### Sub-Agent: Compliance Checker (Sonnet)

Before we wrap up, a sonnet agent verifies:

1. **Initial Request Compliance**
   - ✓ Did we complete what you asked?
   - ✓ Or did requirements change? (was it superseded?)
   - ✗ Or did we go off-track?

2. **Architecture Compliance**
   - ✓ All `deployment_id` fields added where needed?
   - ✓ No SQLite, no multi-user assumptions?
   - ✓ Integration tests, no mocks?
   - ✓ No over-engineering (only requested changes)?

3. **Documentation Compliance**
   - ✓ Frontmatter updated?
   - ✓ Orphaned files linked in docs/index.md?
   - ✓ Playbook compliance check ran?

**Output:**
```
✅ COMPLIANCE REPORT
==================

Initial Request:
✅ "Implement frame comparison tests"
   ✓ 6 unit tests added
   ✓ All test types covered
   ✓ Clippy clean, 100% pass rate

Architecture Compliance:
✅ Northstar Principles
   ✓ deployment_id added to CaptureEvent
   ✓ No SQLite changes
   ✓ Pure Rust (no Python in hot path)
   ✓ Integration tests (image sizes, histogram distance)
   ✓ No mocks (real image objects)

Documentation:
✅ All Updated
   ✓ logging.md frontmatter: 2026-02-19
   ✓ roadmap.md: broken link removed
   ✓ No orphaned files created

Code Quality:
✅ All Checks Pass
   ✓ cargo check: clean
   ✓ clippy: 0 warnings
   ✓ tests: 6/6 passing

Status: READY TO MERGE ✅
```

---

## Stage 7: Final Report

I generate and display:

```
🎉 IMPLEMENTATION COMPLETE
==========================

Committed:
  ✅ 7d15610 - Fix build blockers, add unit tests, and clean up docs

Changes:
  • capture/Cargo.toml (image-compare 0.3 → 0.4)
  • capture/src/frame_comparer.rs (+76 lines: 6 tests)
  • capture/src/pipeline.rs (+1 line: deployment_id field)
  • docs/* (2 fixes)

Tests: 6/6 passing
Warnings: 0
Compliance: ✅ FULL

Next Steps:
  1. Review the PR (if auto-created)
  2. Merge to main
  3. Use /implement again for next task

Ready for the next task? Use /implement again.
```

---

## Complete Workflow Diagram

```
YOUR REQUEST
    ↓
Context Briefing Agent (Haiku) → you pick task
    ↓
Plan Mode (read code, generate plan) → you approve
    ↓
PARALLEL:
  Code Writer (Sonnet) ←→ Test Writer (Haiku)
  Documentation Writer (Haiku)
    ↓
Test/Debug (loop until passing)
    ↓
Your Quick Pass Review
    ↓
Compliance Checker (Sonnet) ← Verifies against:
                               • Initial request
                               • Northstar
                               • Docs playbook
    ↓
Final Report + Ready for Next Task
```

---

## Key Features

✅ **Parallel Execution**: Code + docs + tests run simultaneously
✅ **Sub-Agent Communication**: You can ask follow-up questions → agents adapt
✅ **Clean Context**: Heavy lifting offloaded to sub-agents, you stay focused
✅ **Compliance Verification**: Automatic check against requirements + architecture
✅ **Iterative Debugging**: TDD support with test-first flow
✅ **No Over-Engineering**: Only requested changes, only necessary tests
✅ **Final Sign-Off**: Human review at every stage

---

## Usage

```
/implement
```

That's it. Everything else is orchestrated.

**You stay in control at 3 decision points:**
1. Pick which task to work on
2. Approve the plan
3. Approve the implementation (quick pass)

The rest is automated.
