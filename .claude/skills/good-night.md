---
name: good-night
description: Session summary - what was done, what's next
user-invocable: true
---

# Good Night

Session wrap-up: summarize work, update status, and prepare for tomorrow.

## What Happened Today

Review our session:
1. **What tasks did we complete?** (mark as `completed` in todo.md)
2. **What's still in progress?** (keep status as `in_progress`)
3. **Any blockers or issues?** (note them)

## Update todo.md

Go through `todo.md` and:
- Mark completed tasks as `status: completed`
- Move stalled tasks to `status: in_progress` with a note about the blocker
- Add any new tasks discovered during implementation

## Session Summary

Tell me:
- ✅ Tasks completed (with commit hashes if applicable)
- 🔄 Tasks still in progress (what's blocking them)
- ❓ Any questions for tomorrow
- 🚀 Recommended next tasks (ordered by priority)

## Prepare for Tomorrow

Tomorrow morning, you'll see `/morning` again. That will:
1. Show the morning checklist (SessionStart hook)
2. Review the updated todo.md
3. Pick the next task to implement

**Until tomorrow!**
