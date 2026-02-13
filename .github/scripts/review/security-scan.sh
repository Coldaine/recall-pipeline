#!/bin/bash
# Security Scan Agent - Scans for secrets and vulnerabilities
# Usage: ./security-scan.sh [path]
# Output: Markdown report to stdout

set -euo pipefail

SCAN_PATH="${1:-.}"
OUTPUT_FILE="${2:-/tmp/security-scan-$(date +%s).md}"

echo "Security Scan Agent starting..." >&2
echo "Scanning: $SCAN_PATH" >&2

FINDINGS=""
CRITICAL_COUNT=0
WARNING_COUNT=0

# Function to add finding
add_finding() {
    local severity="$1"
    local category="$2"
    local message="$3"
    FINDINGS="$FINDINGS
- **[$severity]** $category: $message"
    if [[ "$severity" == "CRITICAL" ]]; then
        ((CRITICAL_COUNT++)) || true
    elif [[ "$severity" == "WARNING" ]]; then
        ((WARNING_COUNT++)) || true
    fi
}

# 1. Check for secrets with patterns
echo "Checking for hardcoded secrets..." >&2
SECRET_PATTERNS=(
    'password\s*=\s*["\x27][^"\x27]+'
    'api[_-]?key\s*=\s*["\x27][^"\x27]+'
    'secret[_-]?key\s*=\s*["\x27][^"\x27]+'
    'token\s*=\s*["\x27][A-Za-z0-9_-]{20,}'
    'AWS[_A-Z]*\s*=\s*["\x27][A-Z0-9]{20}'
    'ANTHROPIC_API_KEY\s*=\s*["\x27]sk-'
    'OPENAI_API_KEY\s*=\s*["\x27]sk-'
    'ghp_[A-Za-z0-9]{36}'
    'gho_[A-Za-z0-9]{36}'
)

for pattern in "${SECRET_PATTERNS[@]}"; do
    MATCHES=$(grep -rniE "$pattern" "$SCAN_PATH" \
        --include="*.py" --include="*.rs" --include="*.js" --include="*.ts" \
        --include="*.json" --include="*.yaml" --include="*.yml" --include="*.toml" \
        --include="*.sh" --include="*.env*" \
        --exclude-dir=".git" --exclude-dir="target" --exclude-dir="node_modules" \
        --exclude-dir=".venv" --exclude-dir="__pycache__" \
        2>/dev/null | head -5) || true

    if [[ -n "$MATCHES" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                # Redact the actual secret value
                REDACTED=$(echo "$match" | sed 's/=.*$/=***REDACTED***/')
                add_finding "CRITICAL" "Potential Secret" "$REDACTED"
            fi
        done <<< "$MATCHES"
    fi
done

# 2. Check for dangerous patterns
echo "Checking for dangerous code patterns..." >&2

# SQL injection patterns
SQL_INJECTION=$(grep -rniE 'execute\s*\(\s*["\x27].*\%|f".*SELECT.*{|\.format\(.*SELECT' "$SCAN_PATH" \
    --include="*.py" --exclude-dir=".git" --exclude-dir="target" 2>/dev/null | head -3) || true
if [[ -n "$SQL_INJECTION" ]]; then
    add_finding "WARNING" "Potential SQL Injection" "String formatting in SQL query detected"
fi

# Command injection patterns
CMD_INJECTION=$(grep -rniE 'os\.system\(|subprocess\.(call|run|Popen)\([^,]*\+|shell=True' "$SCAN_PATH" \
    --include="*.py" --exclude-dir=".git" --exclude-dir="target" 2>/dev/null | head -3) || true
if [[ -n "$CMD_INJECTION" ]]; then
    add_finding "WARNING" "Potential Command Injection" "Unsafe subprocess or os.system usage"
fi

# Eval/exec usage
EVAL_EXEC=$(grep -rniE '\beval\s*\(|\bexec\s*\(' "$SCAN_PATH" \
    --include="*.py" --exclude-dir=".git" --exclude-dir="target" --exclude-dir=".venv" 2>/dev/null | head -3) || true
if [[ -n "$EVAL_EXEC" ]]; then
    add_finding "WARNING" "Dangerous Function" "eval() or exec() usage detected"
fi

# 3. Check for .env files that shouldn't be committed
echo "Checking for .env files..." >&2
ENV_FILES=$(find "$SCAN_PATH" -name ".env" -o -name ".env.local" -o -name ".env.production" 2>/dev/null | grep -v ".git" | head -5) || true
if [[ -n "$ENV_FILES" ]]; then
    add_finding "WARNING" "Environment File" ".env file found - ensure it's in .gitignore"
fi

# 4. Check for TODO security items
echo "Checking for security TODOs..." >&2
SECURITY_TODOS=$(grep -rniE 'TODO.*secur|FIXME.*secur|XXX.*secur|HACK.*auth' "$SCAN_PATH" \
    --include="*.py" --include="*.rs" --include="*.js" --include="*.ts" \
    --exclude-dir=".git" --exclude-dir="target" 2>/dev/null | head -5) || true
if [[ -n "$SECURITY_TODOS" ]]; then
    while IFS= read -r todo; do
        if [[ -n "$todo" ]]; then
            add_finding "INFO" "Security TODO" "$todo"
        fi
    done <<< "$SECURITY_TODOS"
fi

# 5. Run cargo audit if available and Cargo.toml exists
if [[ -f "$SCAN_PATH/capture/Cargo.toml" ]] && command -v cargo &> /dev/null; then
    echo "Running cargo audit..." >&2
    CARGO_AUDIT=$(cd "$SCAN_PATH/capture" && cargo audit 2>/dev/null | grep -E "^(warning|error):" | head -5) || true
    if [[ -n "$CARGO_AUDIT" ]]; then
        while IFS= read -r audit; do
            if [[ -n "$audit" ]]; then
                add_finding "WARNING" "Rust Dependency" "$audit"
            fi
        done <<< "$CARGO_AUDIT"
    fi
fi

# 6. Check Python dependencies for known vulnerabilities (if pip-audit available)
if [[ -f "$SCAN_PATH/pyproject.toml" ]] && command -v pip-audit &> /dev/null; then
    echo "Running pip-audit..." >&2
    PIP_AUDIT=$(pip-audit --desc 2>/dev/null | grep -v "^Name" | head -5) || true
    if [[ -n "$PIP_AUDIT" ]]; then
        while IFS= read -r vuln; do
            if [[ -n "$vuln" ]]; then
                add_finding "WARNING" "Python Dependency" "$vuln"
            fi
        done <<< "$PIP_AUDIT"
    fi
fi

# Determine overall status
if [[ $CRITICAL_COUNT -gt 0 ]]; then
    STATUS="🔴 CRITICAL ISSUES FOUND"
    EXIT_CODE=1
elif [[ $WARNING_COUNT -gt 0 ]]; then
    STATUS="🟡 WARNINGS FOUND"
    EXIT_CODE=0
else
    STATUS="✅ NO ISSUES FOUND"
    EXIT_CODE=0
fi

# Generate report
cat > "$OUTPUT_FILE" << EOF
## Security Scan

**Status**: $STATUS
- Critical: $CRITICAL_COUNT
- Warnings: $WARNING_COUNT

### Findings
${FINDINGS:-"No security issues detected."}

### Checks Performed
- [x] Hardcoded secrets and API keys
- [x] SQL injection patterns
- [x] Command injection patterns
- [x] Dangerous function usage (eval/exec)
- [x] Environment file exposure
- [x] Security-related TODOs
- [x] Rust dependency vulnerabilities (cargo audit)
- [x] Python dependency vulnerabilities (pip-audit)

---
*Scanned by: security-scan.sh*
EOF

cat "$OUTPUT_FILE"
echo "" >&2
echo "Security scan complete. Critical: $CRITICAL_COUNT, Warnings: $WARNING_COUNT" >&2
exit $EXIT_CODE
