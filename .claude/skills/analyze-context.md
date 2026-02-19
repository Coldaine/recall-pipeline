---
name: analyze-context
description: Dispatch haiku sub-agents to read docs and GitHub state (used by good-night-extended)
user-invocable: false
---

# Analyze Context (Internal)

This skill is called by `/good-night-extended`. It dispatches two parallel haiku agents.

## Agent 1: Documentation Analyzer

**Prompt to haiku agent:**

```
You are a project analyst for Recall Pipeline.

Read these documents and summarize the CONSTRAINTS and DECISIONS:
1. docs/Northstar.md - Immutable principles
2. docs/architecture/adr-001.md - PostgreSQL decision
3. docs/architecture/adr-002.md - Pure Rust decision
4. docs/architecture/adr-003.md - Capture rewrite decision
5. AGENTS.md - Workflow conventions

For each, extract:
- Core constraint or decision
- Why it matters (consequences)
- What it forbids or requires

Output as:
📋 ARCHITECTURE CONSTRAINTS
[list each with reasoning]

🎯 WORKFLOW PRINCIPLES
[list each with reasoning]
```

## Agent 2: GitHub Intelligence Gatherer

**Prompt to haiku agent:**

```
You have access to GitHub CLI. This is Recall Pipeline (coldaine/recall-pipeline).

Run these commands and analyze the output:

1. gh pr list --state open --json number,title,state,createdAt --limit 10
2. gh issue list --state open --json number,title,labels --limit 15

For PRs, report:
- What's merging soon (≤2 days old)
- What's stale (>1 week old)
- Any blockers or conflicts noted

For issues, report:
- High-priority (label: urgent, blocking, bug)
- Related to capture/storage/agents (by labels)
- Recently opened (last 3 days)

Output as:
🚀 ACTIVE PRS
[list with status and age]

📌 OPEN ISSUES (by priority)
[list critical → important → nice-to-have]

⚠️  BLOCKERS & CONFLICTS
[anything preventing progress]

Recommend merge/rebase order if needed.
```

## Parallel Execution

Both agents run simultaneously. Combine outputs into a single context summary.

## Decision Making

After both reports, the user answers:
1. Should I rebase before committing my work?
2. Are there PRs I should review/merge first?
3. Should my next task change based on blocking issues?
