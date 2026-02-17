# Multi-Agent Review Architecture

## Overview

Comprehensive automated review system using multiple AI agents to analyze code quality, security, architecture compliance, and test coverage on pushes and PRs.

## Design Principles

1. **Tiered Depth**: Fast checks on push, full review on PR, deep audit on merge/schedule
2. **Parallel Execution**: Independent agents run concurrently
3. **Graceful Degradation**: If one agent fails, others continue
4. **Aggregated Reporting**: Single unified report from all agents
5. **Cost Awareness**: Use appropriate models for each task (fast/cheap for simple, powerful for complex)

## Agent Roles

| Agent | Trigger | Purpose | Tool |
|-------|---------|---------|------|
| **Linter** | Push, PR | Code style, formatting | ruff, clippy, cargo fmt |
| **Security Scanner** | Push, PR | Secrets, vulnerabilities | trufflehog, cargo audit, bandit |
| **Code Reviewer** | PR | Logic bugs, anti-patterns, improvements | Claude/Kilocode |
| **Test Analyzer** | PR | Coverage gaps, test quality | pytest-cov + LLM analysis |
| **Architecture Guardian** | PR (core files) | Consistency with design docs | Claude with arch context |
| **Dependency Auditor** | Weekly, dep changes | Outdated/vulnerable deps | pip-audit, cargo audit + LLM |

## Workflow Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PUSH EVENT                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                       │
│  │ Lint (ruff)  │  │ Lint (clippy)│  │ Secret Scan  │   ← Fast, parallel    │
│  │ Python       │  │ Rust         │  │ (trufflehog) │                       │
│  └──────────────┘  └──────────────┘  └──────────────┘                       │
│         │                │                  │                                │
│         └────────────────┴──────────────────┘                                │
│                          │                                                   │
│                    [PASS/FAIL]                                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           PULL REQUEST EVENT                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Fast Checks (same as push)                                                  │
│         │                                                                    │
│         ▼                                                                    │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    PARALLEL AGENT REVIEW                              │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐     │   │
│  │  │ Code Review │ │ Test        │ │ Security    │ │ Architecture│     │   │
│  │  │ Agent       │ │ Analyzer    │ │ Audit       │ │ Guardian    │     │   │
│  │  │ (Kilocode)  │ │ (pytest+LLM)│ │ (LLM)       │ │ (Claude)    │     │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘     │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                          │                                                   │
│                          ▼                                                   │
│                 ┌─────────────────┐                                          │
│                 │   Aggregator    │                                          │
│                 │ (combine reports│                                          │
│                 │  post PR comment│                                          │
│                 └─────────────────┘                                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                         SCHEDULED (Weekly)                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐              │
│  │ Dependency      │  │ Full Security   │  │ Architecture    │              │
│  │ Audit           │  │ Scan            │  │ Drift Report    │              │
│  │ (all deps)      │  │ (full codebase) │  │ (vs design docs)│              │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘              │
│                          │                                                   │
│                          ▼                                                   │
│                 ┌─────────────────┐                                          │
│                 │ Weekly Report   │                                          │
│                 │ (GitHub Issue)  │                                          │
│                 └─────────────────┘                                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Agent Implementation Details

### 1. Code Review Agent

**Tool**: Kilocode (`--auto --json --mode code`)

**Input**: Git diff of changed files

**Prompt Template**:
```
Review this code diff for:
- Logic bugs or edge cases
- Anti-patterns or code smells
- Performance issues
- Missing error handling
- Suggestions for improvement

Provide feedback as markdown with severity levels: 🔴 Critical, 🟡 Warning, 🟢 Suggestion
```

**Output**: Structured markdown review

### 2. Security Audit Agent

**Tool**: Combination of static tools + LLM analysis

**Checks**:
- Secrets in code (trufflehog)
- SQL injection patterns
- Command injection
- XSS vulnerabilities
- Hardcoded credentials
- Insecure dependencies

**Output**: Security findings with CVSS-like severity

### 3. Test Coverage Analyzer

**Tool**: pytest-cov + LLM analysis

**Process**:
1. Run coverage report
2. Identify uncovered lines in changed files
3. LLM suggests test cases for uncovered code

**Output**: Coverage delta + suggested tests

### 4. Architecture Guardian

**Tool**: Claude Code with architecture context

**Input**:
- Changed files
- `docs/architecture/overview.md`
- Design principles

**Checks**:
- Does change follow established patterns?
- Does it violate layer boundaries?
- Does it need architecture doc updates?

**Output**: Architecture compliance report

## Provider Selection Strategy

| Task Complexity | Provider | Rationale |
|-----------------|----------|-----------|
| Simple lint aggregation | Local script | No LLM needed |
| Quick security pattern match | Goose (fast) | Speed, low cost |
| Code review | Kilocode | Good balance of quality/speed |
| Architecture review | Claude (via CCR) | Needs deep reasoning |
| Complex security audit | Claude | Needs context understanding |

## Fallback Chain

Each agent has fallback providers:

```
Primary → Secondary → Tertiary → Skip with warning
```

Example for Code Review:
```
Kilocode → Goose → CCR → Skip (log warning, don't block)
```

## Cost Management

- **Push events**: Fast tools only (free)
- **PR events**: Limited LLM calls (cap at N per PR)
- **Scheduled**: Budget-aware (use cheaper models for bulk)
- **Token limits**: Truncate large diffs, summarize context

## Report Format

```markdown
# AI Review Report

## Summary
- 🔴 Critical: 0
- 🟡 Warnings: 3
- 🟢 Suggestions: 5
- ✅ Checks Passed: 12

## Code Review (Kilocode)
[findings...]

## Security Scan
[findings...]

## Test Coverage
- Coverage: 78% (+2%)
- Uncovered in diff: 15 lines
[suggested tests...]

## Architecture
[compliance notes...]

---
*Generated by multi-agent review suite*
```

## Implementation Phases

### Phase 1: Foundation (Current Sprint)
- [ ] Create review scripts directory
- [ ] Implement code-review-agent.sh
- [ ] Implement security-scan.sh
- [ ] Create report-aggregator.sh
- [ ] Add pr-review.yml workflow

### Phase 2: Enhancement
- [ ] Add test coverage analyzer
- [ ] Add architecture guardian
- [ ] Implement caching for repeated reviews
- [ ] Add review diff (only re-review changed parts)

### Phase 3: Intelligence
- [ ] Smart agent routing based on file types
- [ ] Learning from past reviews (what gets approved)
- [ ] Auto-fix suggestions for common issues
- [ ] Integration with Jules for async fixes

## File Structure

```
.github/
├── workflows/
│   ├── ci.yml              # Existing CI
│   ├── pr-review.yml       # NEW: Multi-agent PR review
│   └── weekly-audit.yml    # NEW: Scheduled deep audit
└── scripts/
    └── review/
        ├── code-review-agent.sh
        ├── security-scan.sh
        ├── test-analyzer.sh
        ├── arch-guardian.sh
        └── report-aggregator.sh
```
