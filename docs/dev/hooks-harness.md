---
last_edited: 2026-02-19
editor: Claude Code (Claude Opus 4.6)
user: Coldaine
status: ready
version: 1.0.0
subsystem: monorepo
tags: [hooks, claude-code, development-harness, operational-guide, tutorial]
doc_type: guide
---

# Claude Code Hooks Operational Guide

**Location**: `.claude/hooks/`
**Configuration**: `.claude/settings.json`
**Reference Architecture**: [ADR-004: Hooks-as-Primitives Development Harness](../architecture/adr-004.md)

This guide is your practical reference for writing, testing, and debugging Claude Code hooks in Recall Pipeline. Use it alongside ADR-004 (theory) to understand when and how to implement development automation.

---

## 1. Quick Start: Write Your First Hook

### Scenario

You want to create a hook that runs on session start to print a custom checklist.

### Step 1: Create the Hook Script

Create a new executable bash script in `.claude/hooks/`:

```bash
#!/bin/bash
set -euo pipefail

# my-checklist.sh
# SessionStart hook for Recall Pipeline
# Prints a custom checklist on session start

cat << 'EOF'

╔════════════════════════════════════════════════════════╗
║  MY CUSTOM CHECKLIST                                   ║
║  ✅ Read docs/Northstar.md                             ║
║  ✅ Check todo.md for blockers                         ║
║  ✅ Review recent commits                              ║
╚════════════════════════════════════════════════════════╝

EOF

# Output JSON for Claude to parse
echo '{"status": "checklist-printed", "timestamp": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"}'
```

Make it executable:

```bash
chmod +x .claude/hooks/my-checklist.sh
```

### Step 2: Register the Hook in `.claude/settings.json`

Add your hook to the appropriate lifecycle event:

```json
{
  "enableAllProjectMcpServers": true,
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume",
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/session-start.sh"
          },
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/my-checklist.sh"
          }
        ]
      }
    ]
  }
}
```

**Key points**:
- `matcher`: Regex pattern on event metadata. Empty string `""` matches all. `"startup|resume"` matches session start or resume only.
- `hooks`: Array of commands to execute (run sequentially in the order listed).
- `$CLAUDE_PROJECT_DIR`: Environment variable pointing to your project root.

### Step 3: Test It

Start a new Claude Code session. Your hook will run automatically and print the checklist.

---

## 2. Hook Structure & Registration

### Lifecycle Events

Claude Code fires these events. Each can trigger one or more hooks:

| Event | When | Typical Use |
|---|---|---|
| **SessionStart** | Session begins or resumes | Load context, print checklists, run diagnostics |
| **Stop** | Claude finishes responding | Auto-save progress, update progress trackers |
| **PreToolUse** | Before tool executes (Bash, Write, Edit, etc.) | Gate destructive operations, enforce conventions |
| **PostToolUse** | After tool completes | Run tests, verify output, update indexes |
| **SessionEnd** | Session terminates | Cleanup, compliance check, archival (unreliable in web) |

### Hook Configuration Schema

In `.claude/settings.json`:

```json
{
  "hooks": {
    "EventName": [
      {
        "matcher": "regex-pattern-or-empty-string",
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/script-name.sh"
          }
        ]
      }
    ]
  }
}
```

**Fields**:
- `matcher` (string): Regex pattern against event metadata. Empty `""` matches all instances.
- `hooks` (array): List of commands to run. Executed sequentially in order.
- `type` (string): Always `"command"` (bash commands).
- `command` (string): Path to executable. Use `$CLAUDE_PROJECT_DIR` for repo-relative paths.

### Example: Multi-Hook Session Start

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/session-start.sh" },
          { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/load-memory.sh" }
        ]
      },
      {
        "matcher": "resume",
        "hooks": [
          { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/session-start.sh" },
          { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/check-progress.sh" }
        ]
      }
    ]
  }
}
```

This configuration:
- On **startup**: Runs `session-start.sh`, then `load-memory.sh`.
- On **resume**: Runs `session-start.sh`, then `check-progress.sh`.

---

## 3. Environment Variables Available in Hooks

When a hook runs, Claude Code provides these environment variables:

| Variable | Content | Example |
|---|---|---|
| `$CLAUDE_PROJECT_DIR` | Absolute path to project root | `/home/user/recall-pipeline` |
| `$CLAUDE_CODE_REMOTE` | `"true"` if web, absent if desktop | `true` |
| `$CLAUDE_CODE_VERSION` | Claude Code version | `1.2.5` |
| `$HOME` | User home directory | `/home/user` |
| `$PATH` | Standard shell PATH | (inherited) |

**Example usage in hooks**:

```bash
#!/bin/bash
# Access project root
cd "$CLAUDE_PROJECT_DIR"

# Check if running in web context
if [ "${CLAUDE_CODE_REMOTE:-}" = "true" ]; then
    echo "Running in Claude Code web environment"
else
    echo "Running in desktop environment"
fi

# Read a project file
TODO_COUNT=$(grep -c "status: pending" "$CLAUDE_PROJECT_DIR/todo.md" || echo 0)
echo "Pending tasks: $TODO_COUNT"
```

---

## 4. The 60-Second Timeout Constraint

**All hooks must complete within 60 seconds.** If a hook exceeds this timeout, Claude Code cancels it and the session continues (behavior varies by event type).

### Implications

- ✅ **Fast operations**: File reads, git status, regex matching, JSON parsing
- ✅ **Async kicks**: Start a background job and immediately exit
- ✅ **External queries**: API calls with short timeouts (<=30s response time)
- ❌ **Long builds**: `cargo build --release` will timeout
- ❌ **Large test suites**: Full `cargo test` will likely timeout
- ❌ **Heavy processing**: Image processing, video transcoding

### Pattern: Async Kick

To run something long-running without blocking:

```bash
#!/bin/bash
set -euo pipefail

# Kick off a background job asynchronously
# The hook exits immediately; the job continues

nohup "$CLAUDE_PROJECT_DIR/.claude/hooks/long-running-job.sh" > /tmp/hook-job.log 2>&1 &

# Report to Claude immediately (well under 60 seconds)
echo '{"status": "background-job-started", "pid": "'$!'"}'
```

### Pattern: Fast Checks Only

Keep hooks focused on speed:

```bash
#!/bin/bash
set -euo pipefail

# Only do lightweight checks in hooks
cd "$CLAUDE_PROJECT_DIR"

# Fast: check file exists
if [ ! -f docs/index.md ]; then
    echo '{"error": "docs/index.md missing"}'
    exit 1
fi

# Fast: count lines
LINES=$(wc -l < docs/index.md)
echo '{"docs_index_size": '$LINES'}'
```

---

## 5. Common Patterns

### Pattern A: State Management Across Sessions

Hooks are stateless (fresh shell each run). For cross-session state:

```bash
#!/bin/bash
set -euo pipefail

# Use the filesystem as a state store
STATE_FILE="$CLAUDE_PROJECT_DIR/.claude/state/session.json"
mkdir -p "$(dirname "$STATE_FILE")"

# Read previous state
if [ -f "$STATE_FILE" ]; then
    PREV_STATE=$(cat "$STATE_FILE")
else
    PREV_STATE='{}'
fi

# Update state
NEW_STATE='{"last_session": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'", "session_count": 1}'

# Persist it
echo "$NEW_STATE" > "$STATE_FILE"

# Report to Claude
echo "$NEW_STATE"
```

**Better approach for structured data**: Use PostgreSQL (Recall Pipeline already uses it):

```bash
#!/bin/bash
set -euo pipefail

# Store state in PostgreSQL
psql -h "${DB_HOST:-localhost}" -U "$DB_USER" -d "$DB_NAME" << SQL
INSERT INTO hook_state (key, value, updated_at)
VALUES ('session_start', '$(date -u +%Y-%m-%dT%H:%M:%SZ)', NOW())
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();
SQL

echo '{"status": "state-persisted"}'
```

### Pattern B: Matcher-Based Branching

Use regex matchers to run different logic based on event metadata:

```bash
#!/bin/bash
set -euo pipefail

# Hook is invoked with event metadata in the environment
# Behavior can differ based on matcher

# Example: different logic for startup vs resume
if [ "${CLAUDE_SESSION_TYPE:-}" = "startup" ]; then
    echo "Full context load (startup)"
    # Run expensive operations
elif [ "${CLAUDE_SESSION_TYPE:-}" = "resume" ]; then
    echo "Quick context load (resume)"
    # Skip expensive operations
fi
```

Better: Use separate hook entries in settings.json with different matchers (see Section 2).

### Pattern C: Matcher Composition

Combine multiple matchers for fine-grained control:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "command=(git|git-commit).*push.*--force",
        "hooks": [
          { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/block-force-push.sh" }
        ]
      },
      {
        "matcher": "command=write.*\\.env",
        "hooks": [
          { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/protect-env.sh" }
        ]
      }
    ]
  }
}
```

### Pattern D: Hook Composition (Calling Other Hooks)

Hooks are just bash scripts. They can call other scripts to compose behavior:

```bash
#!/bin/bash
set -euo pipefail

# main-hook.sh - calls smaller reusable hooks

HOOKS_DIR="$CLAUDE_PROJECT_DIR/.claude/hooks"

# Execute sub-hooks in sequence
"$HOOKS_DIR/check-docs.sh" || exit 1
"$HOOKS_DIR/check-git.sh" || exit 1
"$HOOKS_DIR/check-code.sh" || exit 1

echo '{"status": "all-checks-passed"}'
```

Then in settings.json, just register the main hook:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/main-hook.sh" }
        ]
      }
    ]
  }
}
```

### Pattern E: Output JSON for Claude Integration

Hooks can output JSON to communicate status to Claude:

```bash
#!/bin/bash
set -euo pipefail

# Collect info
PENDING_TASKS=$(grep -c "status: pending" "$CLAUDE_PROJECT_DIR/todo.md" || echo 0)
UNCOMMITTED=$(cd "$CLAUDE_PROJECT_DIR" && git status --porcelain | wc -l)
BRANCH=$(cd "$CLAUDE_PROJECT_DIR" && git rev-parse --abbrev-ref HEAD)

# Output structured JSON
cat << EOF
{
  "status": "session-ready",
  "branch": "$BRANCH",
  "pending_tasks": $PENDING_TASKS,
  "uncommitted_changes": $UNCOMMITTED,
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
```

Claude Code parses this JSON and can use it to customize behavior. Keep JSON on the last line of output (if multiple lines, use `echo` for logging, JSON at the end).

---

## 6. Debugging Hooks

### Symptom: Hook Didn't Run

**Check**:
1. Is `.claude/settings.json` valid JSON? `jq . .claude/settings.json`
2. Is the hook executable? `ls -la .claude/hooks/my-hook.sh` (should show `x`)
3. Does the hook path exist? `file .claude/hooks/my-hook.sh`
4. Did Claude Code reload settings? Restart the session.

### Symptom: Hook Ran but No Output

**Debug**:
1. **Check return code** — Hook exiting with error silently.
2. **Add logging** — Pipe output to a log file:

```bash
#!/bin/bash
set -euo pipefail

# Log to file for debugging
{
    echo "Hook started at $(date)"

    # Your hook logic
    cd "$CLAUDE_PROJECT_DIR"
    echo "Project dir: $CLAUDE_PROJECT_DIR"

    # If error, log it
    if [ ! -f "todo.md" ]; then
        echo "ERROR: todo.md not found"
        exit 1
    fi

    echo "Hook completed"
} >> /tmp/hook-debug.log 2>&1

echo '{"status": "completed"}'
```

Then check the log: `cat /tmp/hook-debug.log`

### Symptom: Hook Timed Out

**Diagnosis**: The hook took >60 seconds.

**Solutions**:
1. **Profile it**: Add timestamps.
   ```bash
   echo "Step 1: $(date)" >&2
   # expensive operation
   echo "Step 2: $(date)" >&2
   ```
2. **Cache expensive results**:
   ```bash
   CACHE_FILE="/tmp/hook-cache-$(whoami).json"
   if [ -f "$CACHE_FILE" ] && [ "$(stat -c %Y "$CACHE_FILE")" -gt "$(date +%s)" - 300 ]; then
       cat "$CACHE_FILE"
   else
       # Expensive operation
       RESULT='...'
       echo "$RESULT" > "$CACHE_FILE"
   fi
   ```
3. **Run async** (Section 4): Start background job and exit immediately.

### Symptom: Hook Logic Is Wrong

**Debug approach**:
1. **Test in isolation** — Run the script directly.
   ```bash
   ./.claude/hooks/my-hook.sh
   ```
2. **Trace execution** — Add `set -x` for debug mode:
   ```bash
   #!/bin/bash
   set -euxo pipefail  # -x prints each command
   ```
3. **Check variables** — Print environment variables in the hook:
   ```bash
   echo "PROJECT_DIR: $CLAUDE_PROJECT_DIR"
   echo "REMOTE: ${CLAUDE_CODE_REMOTE:-unset}"
   ```

---

## 7. Testing Hooks Locally

### Test 1: Syntax Check

```bash
# Check for bash syntax errors
bash -n .claude/hooks/my-hook.sh
```

If no output, syntax is valid.

### Test 2: Direct Execution

```bash
# Run the hook in isolation
export CLAUDE_PROJECT_DIR=/home/user/recall-pipeline
export CLAUDE_CODE_REMOTE=true

./.claude/hooks/my-hook.sh

# Capture exit code
echo "Exit code: $?"
```

### Test 3: Simulate Event Context

Hooks receive metadata from Claude Code. You can simulate this:

```bash
# Simulate SessionStart event
export CLAUDE_PROJECT_DIR=/home/user/recall-pipeline
export CLAUDE_CODE_REMOTE=true
export CLAUDE_SESSION_TYPE=startup

./.claude/hooks/session-start.sh
```

### Test 4: Integration with Settings

Add your hook to `.claude/settings.json`, then start a new Claude Code session. The hook will run automatically.

### Test 5: Capture Output & Exit Code

```bash
#!/bin/bash
# test-hook.sh

OUTPUT=$(./.claude/hooks/my-hook.sh 2>&1)
EXIT_CODE=$?

echo "OUTPUT: $OUTPUT"
echo "EXIT_CODE: $EXIT_CODE"

# Validate JSON output (if your hook outputs JSON)
if command -v jq &> /dev/null; then
    echo "$OUTPUT" | jq . && echo "JSON valid" || echo "JSON invalid"
fi
```

### Test 6: Performance Testing

```bash
#!/bin/bash
# time-hook.sh

time ./.claude/hooks/my-hook.sh

# Or with more detail:
SECONDS=0
./.claude/hooks/my-hook.sh
echo "Execution time: ${SECONDS}s"

# Must complete in <60s
if [ $SECONDS -gt 60 ]; then
    echo "ERROR: Hook exceeded 60-second timeout!"
    exit 1
fi
```

---

## 8. Real-World Examples

### Example 1: SessionStart Checklist (Phase 1)

See `.claude/hooks/session-start.sh`:

```bash
#!/bin/bash
set -euo pipefail

# Only run in web environment
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
    exit 0
fi

# Print morning checklist
cat << 'EOF'
╔════════════════════════════════════════════════════════╗
║  🌅 RECALL PIPELINE - MORNING STANDUP                 ║
║  Read context → pick task → implement                 ║
╚════════════════════════════════════════════════════════╝
EOF

# Report status
echo '{"status": "morning-checklist-ready"}'
```

**How it works**:
- Checks if running in web context (not desktop).
- Prints formatted checklist.
- Outputs JSON status.
- Registered in `.claude/settings.json` under `SessionStart` event.

### Example 2: PreToolUse Gate (Phase 3 - Planned)

Blocks destructive git operations:

```bash
#!/bin/bash
set -euo pipefail

# Prevent: git reset --hard, git push --force, git clean -f

COMMAND="${1:-}"

if [[ "$COMMAND" =~ git.*reset.*hard|git.*push.*force|git.*clean.*-f ]]; then
    echo '{"error": "Destructive git operation blocked. Confirm with user.", "blocked_command": "'$COMMAND'"}'
    exit 1
fi

echo '{"status": "operation-allowed"}'
```

**Usage**: Register as `PreToolUse` hook with `matcher` targeting git commands.

### Example 3: PostToolUse Auto-Test (Phase 4 - Planned)

Automatically run tests after Rust file edits:

```bash
#!/bin/bash
set -euo pipefail

# Trigger on file edits under capture/src
EDITED_FILE="${1:-}"

if [[ "$EDITED_FILE" =~ capture/src/.*\.rs$ ]]; then
    echo '{"action": "auto-test-triggered", "file": "'$EDITED_FILE'"}'

    # Don't block — just notify. Async testing happens in background
    nohup bash -c 'cd "$CLAUDE_PROJECT_DIR" && cargo test --lib' > /tmp/auto-test.log 2>&1 &
fi
```

---

## 9. Linking Back to Architecture & Code

This section bridges the operational guide to the canonical sources:

### References

- **[ADR-004: Hooks-as-Primitives Architecture](../architecture/adr-004.md)** — Theoretical foundation. Read this to understand the design decisions, trade-offs, and roadmap.

- **Hook Scripts** (`.claude/hooks/`):
  - [`.claude/hooks/session-start.sh`](../../.claude/hooks/session-start.sh) — Phase 1: Session boundary hook (current)
  - [`.claude/hooks/session-end.sh`](../../.claude/hooks/session-end.sh) — Phase 1: Session cleanup (current)

- **Skills** (`.claude/skills/`):
  - [`.claude/skills/morning.md`](../../.claude/skills/morning.md) — Morning standup workflow (integrates with SessionStart hook)
  - [`.claude/skills/implement.md`](../../.claude/skills/implement.md) — Master implementation workflow (uses hooks for context & compliance)
  - [`.claude/skills/next-task.md`](../../.claude/skills/next-task.md) — Task selection workflow

- **Configuration**:
  - [`.claude/settings.json`](../../.claude/settings.json) — Hook registration and lifecycle event mapping

### When to Read What

| You Want To... | Read This |
|---|---|
| Understand why hooks exist | [ADR-004](../architecture/adr-004.md) → Context & Decision sections |
| Write a new hook | This guide (§1–5) → real-world examples (§8) |
| Debug a hook | This guide (§6) → test locally (§7) |
| Extend hooks to PreToolUse/PostToolUse | [ADR-004](../architecture/adr-004.md) → Phase 3–4 roadmap |
| See a working hook in action | `.claude/hooks/session-start.sh` + `.claude/settings.json` |

---

## 10. Troubleshooting Checklist

Use this table to diagnose common issues:

| Issue | Diagnosis | Fix |
|---|---|---|
| Hook not running | Settings.json invalid? Hook not executable? | `jq . .claude/settings.json` + `chmod +x .claude/hooks/...` |
| Hook hangs | Exceeds 60-second timeout? | Profile with timestamps; use async patterns |
| Hook output not visible | JSON at end of output? Multiple `echo` calls? | Ensure JSON is on final line; use `>&2` for logging |
| Wrong logic | Matcher not matching? Environment variables wrong? | Test in isolation with `export`; print variables |
| State lost between sessions | Using shell variables? | Use filesystem or PostgreSQL (see Pattern A) |
| Hook interferes with normal flow | Exiting with error? Blocking other commands? | Ensure `exit 0` on success; use async for long operations |

---

## Summary

Hooks are simple shell scripts triggered by Claude Code lifecycle events. They:
- ✅ Live in `.claude/hooks/`
- ✅ Register in `.claude/settings.json`
- ✅ Run within 60 seconds
- ✅ Receive environment variables from Claude Code
- ✅ Output JSON to communicate with Claude
- ✅ Compose easily (call other scripts)
- ✅ Persist state via filesystem or database

Start with SessionStart (Phase 1) to load context and print checklists. As you gain experience, extend to PreToolUse (Phase 3) for safety gates and PostToolUse (Phase 4) for feedback loops.

For deeper understanding, read [ADR-004](../architecture/adr-004.md) alongside this guide. For hands-on examples, examine `.claude/hooks/session-start.sh` and adapt it for your needs.
