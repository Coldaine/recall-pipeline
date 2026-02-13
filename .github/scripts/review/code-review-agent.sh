#!/bin/bash
# Code Review Agent - Reviews git diff using LLM providers
# Usage: ./code-review-agent.sh [base_ref] [head_ref]
# Output: Markdown review to stdout, status to stderr

set -euo pipefail

BASE_REF="${1:-origin/main}"
HEAD_REF="${2:-HEAD}"
OUTPUT_FILE="${3:-/tmp/code-review-$(date +%s).md}"

# Source environment
if [[ -f ~/.bashrc ]]; then
    source ~/.bashrc 2>/dev/null || true
fi

echo "Code Review Agent starting..." >&2
echo "Comparing: $BASE_REF..$HEAD_REF" >&2

# Get the diff
DIFF=$(git diff "$BASE_REF".."$HEAD_REF" --no-color 2>/dev/null || git diff "$BASE_REF" "$HEAD_REF" --no-color 2>/dev/null || echo "")

if [[ -z "$DIFF" ]]; then
    echo "No changes to review" >&2
    echo "## Code Review

✅ No code changes detected.
" > "$OUTPUT_FILE"
    cat "$OUTPUT_FILE"
    exit 0
fi

# Truncate diff if too large (keep first 8000 chars)
if [[ ${#DIFF} -gt 8000 ]]; then
    echo "Warning: Diff truncated from ${#DIFF} to 8000 chars" >&2
    DIFF="${DIFF:0:8000}

[... diff truncated for length ...]"
fi

# Get changed file list
CHANGED_FILES=$(git diff --name-only "$BASE_REF".."$HEAD_REF" 2>/dev/null || git diff --name-only "$BASE_REF" "$HEAD_REF" 2>/dev/null || echo "unknown")

# Build prompt
PROMPT="You are an expert code reviewer. Review this git diff and provide feedback.

## Changed Files
$CHANGED_FILES

## Diff
\`\`\`diff
$DIFF
\`\`\`

## Review Instructions
Analyze the code changes for:
1. **Bugs**: Logic errors, edge cases, null checks
2. **Security**: Injection, auth issues, secrets
3. **Performance**: Inefficient patterns, N+1 queries
4. **Maintainability**: Code smells, complexity, naming
5. **Best Practices**: Error handling, logging, testing

## Output Format
Use this exact format:

### Summary
[1-2 sentence overview]

### Findings

#### 🔴 Critical (must fix)
- [finding or 'None']

#### 🟡 Warnings (should fix)
- [findings or 'None']

#### 🟢 Suggestions (nice to have)
- [findings or 'None']

### Verdict
[APPROVE / REQUEST_CHANGES / COMMENT]

Be concise. Focus on the most important issues."

# Try Kilocode first (use --mode ask for text response, not --mode code which does file edits)
try_kilocode() {
    if command -v kilocode &> /dev/null && command -v jq &> /dev/null; then
        echo "Using Kilocode for review..." >&2
        RESPONSE=$(kilocode --auto --json --timeout 120 --mode ask "$PROMPT" 2>/dev/null | sed 's/\x1b\[[0-9;]*[a-zA-Z]//g') || return 1
        # Get the last completion_result with partial:false and content
        RESULT=$(echo "$RESPONSE" | grep '"say":"completion_result"' | grep '"partial":false' | jq -r '.content // empty' | tail -1) || return 1
        if [[ -n "$RESULT" ]]; then
            echo "$RESULT"
            return 0
        fi
    fi
    return 1
}

# Try Goose as fallback
try_goose() {
    if command -v goose &> /dev/null; then
        echo "Using Goose for review..." >&2
        RESULT=$(OPENAI_API_KEY="${OPENAI_API_KEY:-$ZAI_API_KEY}" \
            OPENAI_HOST="${OPENAI_HOST:-https://api.z.ai}" \
            OPENAI_BASE_PATH="${OPENAI_BASE_PATH:-api/coding/paas/v4/chat/completions}" \
            goose run --provider openai --model glm-4.6 --quiet --no-session -t "$PROMPT" 2>/dev/null) || return 1
        if [[ -n "$RESULT" ]]; then
            echo "$RESULT"
            return 0
        fi
    fi
    return 1
}

# Try CCR as last resort
try_ccr() {
    if curl -s "http://127.0.0.1:3456/health" &>/dev/null && command -v jq &>/dev/null; then
        echo "Using CCR for review..." >&2
        RESPONSE=$(curl -s -X POST "http://127.0.0.1:3456/v1/messages" \
            -H "Content-Type: application/json" \
            -H "anthropic-version: 2023-06-01" \
            -H "x-api-key: code-review" \
            -d "{
                \"model\": \"code-review\",
                \"messages\": [{\"role\": \"user\", \"content\": $(echo "$PROMPT" | jq -Rs .)}],
                \"max_tokens\": 2000
            }" 2>/dev/null) || return 1
        RESULT=$(echo "$RESPONSE" | jq -r '.content[0].text // empty') || return 1
        if [[ -n "$RESULT" ]]; then
            echo "$RESULT"
            return 0
        fi
    fi
    return 1
}

# Execute with fallback chain
REVIEW=""
if REVIEW=$(try_kilocode); then
    PROVIDER="Kilocode"
elif REVIEW=$(try_goose); then
    PROVIDER="Goose"
elif REVIEW=$(try_ccr); then
    PROVIDER="CCR"
else
    PROVIDER="None (fallback)"
    REVIEW="## Code Review

⚠️ **Automated review unavailable** - No LLM provider responded.

### Changed Files
$CHANGED_FILES

### Manual Review Required
Please review the changes manually.

---
*Review attempted but no provider available*"
fi

# Write output
cat > "$OUTPUT_FILE" << EOF
## Code Review

$REVIEW

---
*Reviewed by: $PROVIDER*
EOF

cat "$OUTPUT_FILE"
echo "" >&2
echo "Review complete. Provider: $PROVIDER" >&2
echo "Output: $OUTPUT_FILE" >&2
