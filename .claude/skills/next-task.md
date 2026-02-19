---
name: next-task
description: Pick the next incomplete task from todo.md and implement it
user-invocable: true
---

# Next Task

Grab the next incomplete task and implement it.

## Workflow

1. **Find next task** from `todo.md` with status `pending` or `in_progress`
2. **Read context** (Northstar.md, relevant domain docs)
3. **Plan** the change in Plan Mode
4. **Implement** (don't over-engineer)
5. **Test** (cargo check, clippy, cargo test)
6. **Commit** with message including session URL
7. **Ask**: "Should I continue to the next task?"

## Important Reminders

- **Read before changing**: Always read the file/code first
- **Northstar compliance**: deployment_id on tables, no SQLite, pure Rust capture
- **Testing**: Integration tests with real deps, not mocks
- **Documentation**: Update frontmatter if you edit docs
- **Over-engineering**: Only make requested changes
- **Staging rule**: Never delete without explicit final confirmation

Use `/morning` if you need to review architecture context first.
