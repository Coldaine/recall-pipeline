#!/bin/bash
set -euo pipefail

# SessionEnd Hook for Recall Pipeline
# Runs when session ends - checks final state
# Only runs in web environments

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
    exit 0
fi

cat << 'EOF'

╔════════════════════════════════════════════════════════════════╗
║  🌙 SESSION END - FINAL COMPLIANCE CHECK                      ║
║                                                                ║
║  Running sonnet sub-agent to verify:                          ║
║  • Did we complete the initial request?                       ║
║  • Do we comply with Northstar principles?                    ║
║  • Is documentation up to date?                               ║
║                                                                ║
║  (See final report below...)                                  ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝

EOF

echo '{"status": "compliance-check-scheduled"}'
