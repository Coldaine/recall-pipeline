---
name: good-night-extended
description: Extended evening wrap-up - spawn sub-agents to gather project context
user-invocable: true
---

# Good Night Extended

Advanced session wrap-up that gathers project intelligence before you sign off.

## Workflow

This spawns TWO parallel haiku sub-agents to gather context:

### Agent 1: Documentation Context Reader
- **Task**: Summarize recent decisions from:
  - `docs/Northstar.md` (immutable principles)
  - `docs/architecture/adr-*.md` (all ADRs)
  - `AGENTS.md` / `CLAUDE.md` (workflow conventions)
- **Output**: "Based on our principles, here's what matters:"
  - Key constraints affecting this PR/task
  - Recent ADR decisions that impact ongoing work
  - Recommended patterns for this kind of change

### Agent 2: GitHub Context Gatherer
- **Task**: Use GitHub CLI to gather:
  ```bash
  gh pr list --state open --limit 10
  gh issue list --state open --limit 10
  gh issue view <issue-number> (for selected issues)
  ```
- **Output**: Structured analysis:
  - Active PRs and their status
  - Open issues blocking or related to current work
  - Recommended rebase/merge order
  - Does your work conflict with or relate to any PRs?

## Decision Framework

After both agents report, answer:

1. **Do I need to rebase?**
   - Are there open PRs that should merge first?
   - Will rebasing now save time tomorrow?

2. **Are there blocking issues?**
   - Does my work unblock anything?
   - Should I prioritize differently?

3. **What should I do next session?**
   - Recommended task order
   - Any urgent reviews needed?

## Output Format

```
📋 SESSION SUMMARY
==================

✅ Completed:
  - Task 1
  - Task 2

🔄 In Progress:
  - Task 3 (blocker: X)

📊 PROJECT INTELLIGENCE
=======================

Recent Decisions:
  • ADR-001: PostgreSQL only
  • Principle: No mocks, integration tests only

Active PRs:
  • #42: Feature X (ready to merge)
  • #51: Docs update (needs review)

Related Issues:
  • #38: Schema migration (BLOCKING)
  • #45: Performance optimization (nice-to-have)

Recommendations:
  ✓ Merge #42 before starting new tasks
  ✓ Review #51 this session
  ✓ Address #38 blocking issue first tomorrow

Rebase Status:
  ✓ Your branch is up-to-date with main
  ✓ No conflicts detected
```

## Implementation Notes

These sub-agents are haiku (fast, cheap, precise). They:
- Read documentation & GitHub state
- Produce structured, actionable output
- Don't implement anything (pure analysis)
- Run in parallel for speed
- Report back to you for decision-making

Use `/good-night` for simple wrap-up, or `/good-night-extended` for full context.
