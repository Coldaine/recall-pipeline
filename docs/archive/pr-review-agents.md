# Plan: Multi-Agent PR Review System

## Overview

Automated code review system that runs multiple AI agents in parallel when a PR is opened or updated, posting an aggregated review comment to the PR.

## Scope

- **In scope**: Pull requests to `main` branch only
- **Out of scope**: Push events, scheduled reviews, other branches

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     PR Created/Updated                          │
│                            │                                    │
│                            ▼                                    │
│              GitHub fires `pull_request` event                  │
│                            │                                    │
│                            ▼                                    │
│         GitHub Actions triggers on self-hosted runner           │
│                    (laptop-extra-recall)                        │
│                            │                                    │
│                            ▼                                    │
│    ┌──────────────────────────────────────────────────────┐    │
│    │              PARALLEL EXECUTION                       │    │
│    │                                                       │    │
│    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │    │
│    │  │ Kilocode    │  │ Goose       │  │ ruff        │   │    │
│    │  │ (code       │  │ (security   │  │ (lint)      │   │    │
│    │  │  review)    │  │  audit)     │  │             │   │    │
│    │  └─────────────┘  └─────────────┘  └─────────────┘   │    │
│    │                                                       │    │
│    │                    ┌─────────────┐                    │    │
│    │                    │ pytest      │                    │    │
│    │                    │ (tests)     │                    │    │
│    │                    └─────────────┘                    │    │
│    └──────────────────────────────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│                   Aggregate Results                             │
│                            │                                    │
│                            ▼                                    │
│              gh pr comment (create or update)                   │
│                            │                                    │
│                            ▼                                    │
│                  Review visible on PR                           │
└─────────────────────────────────────────────────────────────────┘
```

## Agent Specifications

### 1. Code Review Agent (Kilocode)

**Tool**: `kilocode --auto --json --timeout 90 --mode code`

**Purpose**: Find bugs, logic errors, code smells, improvement opportunities

**Prompt**:
```
Review this PR diff for a Rust/Python codebase.

Focus on:
1. Logic bugs and edge cases
2. Error handling gaps
3. Performance issues
4. Code clarity and maintainability

Changed files: {file_list}

Diff:
{diff}

Output format:
### Findings
- 🔴 **Critical**: [issue] (file:line if known)
- 🟡 **Warning**: [issue]
- 🟢 **Suggestion**: [improvement]

### Summary
[1-2 sentence overall assessment]
```

**Output parsing**: Extract from JSON `completion_result.content`

**Timeout**: 90 seconds

**Fallback**: Skip with warning if fails

### 2. Security Audit Agent (Goose)

**Tool**: `goose run --provider openai --model glm-4.6 --quiet --no-session`

**Environment**:
```bash
OPENAI_API_KEY="${ZAI_API_KEY}"
OPENAI_HOST="https://api.z.ai"
OPENAI_BASE_PATH="api/coding/paas/v4/chat/completions"
```

**Purpose**: Identify security vulnerabilities, secrets, injection risks

**Prompt**:
```
Security audit this PR diff.

Check for:
1. Hardcoded secrets, API keys, tokens
2. SQL/command injection vulnerabilities
3. Unsafe deserialization
4. Authentication/authorization issues
5. Path traversal risks

Diff:
{diff}

Output format:
### Security Findings
- 🔴 **CRITICAL**: [vulnerability] - [remediation]
- 🟡 **WARNING**: [risk] - [recommendation]
- ✅ **PASSED**: [check name]

If no issues found, state "No security issues identified."
```

**Timeout**: 60 seconds

**Fallback**: Skip with warning if fails

### 3. Lint Check (ruff)

**Tool**: `uv run ruff check --output-format=concise`

**Purpose**: Code style, formatting, common errors

**No LLM required** - fast, deterministic

**Output**: Parse ruff output, count errors/warnings

### 4. Test Check (pytest)

**Tool**: `uv run pytest tests/ -v --tb=short -q`

**Purpose**: Verify tests pass

**No LLM required**

**Output**: Pass/fail status, failure details if any

## Execution Flow

### Step 1: Checkout with history
```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0  # Need full history for diff
```

### Step 2: Get diff
```bash
git diff origin/${{ github.base_ref }}...${{ github.sha }} > /tmp/pr.diff
```

### Step 3: Run agents in parallel
```bash
# Background all agents
kilocode_review &
PID1=$!

goose_security &
PID2=$!

ruff_lint &
PID3=$!

pytest_tests &
PID4=$!

# Wait for all
wait $PID1 $PID2 $PID3 $PID4
```

### Step 4: Aggregate results
Combine outputs into single markdown report with:
- Summary table (critical/warning/suggestion counts)
- Section per agent
- Timestamp and commit SHA

### Step 5: Post comment
```bash
# Check for existing bot comment
EXISTING=$(gh pr view $PR_NUMBER --json comments --jq '.comments[] | select(.body | contains("<!-- AI-REVIEW-BOT -->")) | .id' | head -1)

if [ -n "$EXISTING" ]; then
    gh api repos/{owner}/{repo}/issues/comments/$EXISTING -X PATCH -f body="$REPORT"
else
    gh pr comment $PR_NUMBER --body "$REPORT"
fi
```

Use HTML comment marker `<!-- AI-REVIEW-BOT -->` to identify bot comments.

## Comment Format

```markdown
<!-- AI-REVIEW-BOT -->
# 🤖 AI Review

**Commit**: `abc1234`
**Verdict**: ✅ LGTM / ⚠️ Needs attention / 🔴 Changes requested

| Check | Status | Issues |
|-------|--------|--------|
| Code Review | ✅ | 0 critical, 2 suggestions |
| Security | ✅ | No issues |
| Lint | ⚠️ | 3 warnings |
| Tests | ✅ | All passed |

<details>
<summary>📝 Code Review (Kilocode)</summary>

[agent output here]

</details>

<details>
<summary>🔒 Security Audit (Goose)</summary>

[agent output here]

</details>

<details>
<summary>🧹 Lint (ruff)</summary>

[agent output here]

</details>

<details>
<summary>🧪 Tests (pytest)</summary>

[agent output here]

</details>

---
*Review generated at 2025-11-29T12:00:00Z*
```

## File Structure

```
.github/
├── workflows/
│   └── ci.yml                    # Add ai-review job
└── scripts/
    └── review/
        ├── pr-review.sh          # Main orchestrator
        ├── agents/
        │   ├── code-review.sh    # Kilocode wrapper
        │   └── security.sh       # Goose wrapper
        └── templates/
            └── comment.md        # Comment template
```

## GitHub Workflow Addition

Add to `.github/workflows/ci.yml`:

```yaml
  ai-review:
    name: AI Code Review
    runs-on: [self-hosted, Linux, X64, recall-pipeline]
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Run PR Review
        env:
          GH_TOKEN: ${{ github.token }}
          PR_NUMBER: ${{ github.event.pull_request.number }}
          BASE_REF: ${{ github.base_ref }}
          HEAD_SHA: ${{ github.sha }}
        run: |
          chmod +x .github/scripts/review/pr-review.sh
          .github/scripts/review/pr-review.sh
```

## Cost Estimate

Per PR review:
- Kilocode: 1 call
- Goose: 1 call (uses Z.ai GLM-4.6)
- ruff: 0 (local)
- pytest: 0 (local)

**Total**: ~2 LLM calls per PR push

With 5000 Z.ai calls / 5 hours, can review ~2500 PR updates per 5-hour period.

## Error Handling

| Scenario | Handling |
|----------|----------|
| Kilocode fails/times out | Skip, note in report |
| Goose fails/times out | Skip, note in report |
| ruff not installed | Skip lint section |
| Tests fail | Report failures, don't block |
| gh auth fails | Fail workflow (can't post) |
| No diff | Post "No changes to review" |

## Success Criteria

1. PR opened → comment appears within 2 minutes
2. PR updated → comment updates (not new comment)
3. All agents run in parallel (total time < longest agent)
4. Report is readable and actionable
5. No secrets leaked in prompts/outputs

## Implementation Order

1. [ ] Create `.github/scripts/review/` directory structure
2. [ ] Implement `code-review.sh` (Kilocode wrapper)
3. [ ] Implement `security.sh` (Goose wrapper)
4. [ ] Implement `pr-review.sh` (orchestrator)
5. [ ] Add `ai-review` job to `ci.yml`
6. [ ] Test on a real PR
7. [ ] Iterate on prompts based on output quality

## Open Questions

1. Should critical security findings fail the check (block merge)?
2. Should we add architecture review agent later?
3. Rate limiting if many PRs opened quickly?
