#!/bin/bash
# Beads session hook - injects workflow context via bd prime
# Called by SessionStart and PreCompact hooks

if ! command -v bd &> /dev/null; then
    cat >&2 << 'EOF'
WARNING: beads (bd) is not installed.

This project uses beads for AI-agent task tracking. Without it,
you won't have access to the issue database, dependency graph,
or persistent task context across sessions.

Install it:
  brew install beads          # macOS/Linux (Homebrew)
  npm install -g @beads/bd    # npm
  curl -fsSL https://raw.githubusercontent.com/steveyegge/beads/main/scripts/install.sh | bash

Then run:
  bd init        # in this repo (re-creates local SQLite cache from issues.jsonl)

See: https://github.com/steveyegge/beads
EOF
    exit 0
fi

bd prime
