# Justfile for Recall Pipeline Documentation

default:
    @just --list

# Validate documentation structure and naming
validate-docs:
    @echo "Running documentation validation..."
    # TODO: Add python script call here once created
    @echo "Checking for required root files..."
    @if not exist "MasterDocumentationPlaybook.md" (echo "MISSING: MasterDocumentationPlaybook.md" && exit 1)
    @if not exist "Gemini.md" (echo "MISSING: Gemini.md" && exit 1)
    @if not exist "Agent.md" (echo "MISSING: Agent.md" && exit 1)
    @echo "Validation Passed."

# Sync tasks (placeholder for future automation)
sync-tasks:
    @echo "Syncing tasks from todo.md..."

# Maintain docs (full check)
maintain-docs: validate-docs
    @echo "Documentation maintenance complete."
