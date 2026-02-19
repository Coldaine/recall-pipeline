#!/bin/bash
set -euo pipefail

# SessionStart Hook for Recall Pipeline
# Enforces morning workflow: read docs → pick task → implement

# Only run in web environments (Claude Code on the web)
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
    exit 0
fi

# Print morning checklist
cat << 'EOF'

╔════════════════════════════════════════════════════════════════╗
║  🌅 RECALL PIPELINE - MORNING STANDUP                         ║
║                                                                ║
║  You are an autonomous AI software engineer.                   ║
║  Your role: Senior Rust/Python Engineer on Recall Pipeline    ║
║                                                                ║
╠════════════════════════════════════════════════════════════════╣
║                                                                ║
║  📋 MORNING WORKFLOW (Mandatory Pre-Task):                    ║
║                                                                ║
║  1️⃣  READ CONTEXT                                             ║
║      → /morning         (triggers full pre-task workflow)     ║
║      → docs/Northstar.md (immutable principles)               ║
║      → docs/index.md    (architecture & links)                ║
║      → todo.md          (task backlog)                        ║
║                                                                ║
║  2️⃣  PICK NEXT TASK                                          ║
║      → Check todo.md for incomplete items                     ║
║      → Use /next-task to start implementation                 ║
║                                                                ║
║  3️⃣  IMPLEMENT CAREFULLY                                      ║
║      → Read code before changing it                           ║
║      → Don't over-engineer (only requested changes)           ║
║      → Run tests & clippy after changes                       ║
║      → Use Plan Mode for complex tasks                        ║
║                                                                ║
║  4️⃣  WHEN IN DOUBT → ASK                                      ║
║      → Stop and ask clarifying questions                      ║
║      → Never guess about user intent                          ║
║      → Confirm before pushing or deleting                     ║
║                                                                ║
║  🚀 QUICK START:                                              ║
║      /morning      – Full pre-task checklist                  ║
║      /next-task    – Pick & implement one task               ║
║      /good-night   – Session summary & cleanup                ║
║                                                                ║
║  ⚠️  KEY RULES:                                               ║
║      • deployment_id on every table (Northstar §5)            ║
║      • No SQLite, no multi-user tenancy (Northstar §1)        ║
║      • Integration tests, not mocks (Northstar §2)            ║
║      • Never delete without explicit user confirmation        ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝

EOF

# Report hook status to Claude
echo '{"status": "morning-checklist-ready"}'
