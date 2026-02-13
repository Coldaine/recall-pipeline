---
description: Write a session log documenting work accomplished
allowed-tools: Read, Write, Bash, Glob
---

Write a session log for this conversation. Create a markdown file in `docs/session-logs/` with today's date and a brief summary slug.

## File naming
Format: `YYYY-MM-DD-<brief-slug>.md` (e.g., `2025-11-25-ci-setup.md`)

## Log structure

```markdown
# Session Log: <Date>

## Summary
<2-3 sentence summary of what was accomplished>

## Work Completed
- <bullet points of completed work>

## Decisions Made
- <key decisions and their rationale>

## Files Changed
- <list of key files created/modified>

## Open Items
- <anything left incomplete or needing follow-up>

## Next Steps
- <recommended next actions>

---
*Session logged by Claude Code*
```

## Instructions

1. Review the conversation to understand what was accomplished
2. Create the log file with appropriate content
3. Report the file path when done

Do NOT ask for confirmation - just write the log based on the conversation context.
