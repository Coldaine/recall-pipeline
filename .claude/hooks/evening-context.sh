#!/bin/bash
set -euo pipefail

# Evening Context Hook for Recall Pipeline
# Runs at session end or on /good-night to gather project state
# Only runs in web environments

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
    exit 0
fi

cat << 'EOF'

╔════════════════════════════════════════════════════════════════╗
║  🌙 GOOD NIGHT - GATHERING PROJECT CONTEXT                   ║
║                                                                ║
║  Preparing context for tomorrow:                              ║
║  • Your recent decisions (Supermemory)                        ║
║  • Outstanding PRs & issues (GitHub)                          ║
║  • Recommendations for next session                           ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝

EOF

# Report that hook is gathering context
echo '{"status": "evening-context-gathering"}'
