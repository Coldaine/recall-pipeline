---
doc_type: plan
subsystem: general
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2026-02-09
---
# Schema Unification Plan

> **2026-02-09 Note:** The screenpipe-* crates referenced in Track A below have been fully removed and replaced with `recall-capture`, `recall-db`, and `recall-store`. The Track A work (deployment_id migration) has been incorporated into the new crates. Tracks B-E remain accurate. Issue #44 (SQLite rename) is now moot — SQLite has been removed entirely.

**Branch**: `schema-unification`
**PR**: #43
**Status**: In Progress - Awaiting CI/billing resolution
**Created**: 2025-11-17
**Last Updated**: 2025-11-27

---

## Executive Summary

Code unification to bring Rust capture layer and Python agent layer into compliance with the documented single-user, multi-deployment architecture. The work is substantially complete with 5 tracks finished:

- **Track A (Rust)**: `deployment_id` migration complete
- **Track B (Python Memory)**: Multi-tenant → single-user migration complete
- **Track C (Hierarchy)**: Activity/Project/DaySummary tables created
- **Track D (Secrets)**: Table created (encryption pending)
- **Track E (CI)**: GitHub Actions pipeline implemented

**Current Blockers**: CI billing suspension (resolves end of month). Four follow-up issues remain for polish/completeness.

---

## Background and Goals

### Project Purpose

From `docs/architecture/overview.md`:

> **Total digital recall + perfect AI context.** Capture everything across all machines, summarize intelligently at multiple levels, make it searchable and usable as context for LLMs.

### Operating Model

> **Single-user, multi-deployment.** One person, multiple machines (laptop, desktop, work PC). Data flows from deployments to a central aggregation server. No multi-user tenancy.

This is the fundamental shift this branch implements:

- **Was**: Multi-tenant model with `organization_id` and `user_id` on every table
- **Now**: Single-user model with `deployment_id` to identify which machine captured data

### Key Architecture Principles

From `docs/domains/storage.md`:

1. **`deployment_id` on everything** - Identifies which machine captured the data
2. **No `user_id` or `organization_id`** - Single-user system, no multi-tenancy
3. **Hierarchical summaries** - frames → activities → projects → day_summaries
4. **Secret vault with FIDO2** - Hardware key required to decrypt sensitive data
5. **Status codes** - 0=pending, 1=done, 2=error, 3=skipped

---

## Implementation Status

### Completed Work

#### Track A: Rust Schema Normalization ✓
**Scope**: `capture/screenpipe-db/`, `capture/screenpipe-*/`

Completed tasks:
1. Renamed `device_name` → `deployment_id` in all Rust code
2. Updated PostgreSQL migrations to use `deployment_id`
3. Updated `upgrade_schema_extensions.sql` to use `deployment_id`
4. Added `activity_id UUID` column to frames table (nullable)
5. Grepped all Rust crates for `device_name` references and updated

Files modified:
- `capture/screenpipe-db/src/migrations_pg/*.sql`
- `capture/screenpipe-db/src/*.rs`
- `capture/screenpipe-vision/src/*.rs`
- `capture/screenpipe-audio/src/*.rs`
- `capture/screenpipe-core/src/*.rs`

#### Track B: Python Schema Simplification ✓
**Scope**: `agents/orm/`, `agents/schemas/`, `agents/database/`

Completed tasks:
1. Created `DeploymentMixin` to replace `OrganizationMixin` + `UserMixin`
2. Updated all memory ORM models to use `DeploymentMixin`:
   - `episodic_memory.py`
   - `semantic_memory.py`
   - `procedural_memory.py`
   - `resource_memory.py`
   - `knowledge_vault.py`
3. Updated `agents/orm/mixins.py` with `DeploymentMixin`
4. Updated Pydantic schemas in `agents/schemas/`
5. Created `migrate_database_postgresql.sql` for org/user → deployment migration

**Note**: MIRIX models (`Agent`, `Block`, `Message`, etc.) intentionally retain `organization_id`/`user_id`. These are experimental components with a separate migration path (see Issue #47).

Files modified:
- `agents/orm/mixins.py` (added DeploymentMixin)
- `agents/orm/episodic_memory.py`
- `agents/orm/semantic_memory.py`
- `agents/orm/procedural_memory.py`
- `agents/orm/resource_memory.py`
- `agents/orm/knowledge_vault.py`
- `agents/orm/__init__.py`
- `agents/orm/__all__.py`
- `agents/schemas/*.py`
- `agents/database/migrate_database_postgresql.sql`

Files preserved (MIRIX subsystem):
- `agents/orm/organization.py`
- `agents/orm/user.py`
- `agents/orm/agent.py`
- `agents/orm/block.py`
- `agents/orm/message.py`
- `agents/orm/tool.py`
- `agents/orm/file.py`
- `agents/orm/provider.py`
- `agents/orm/sandbox_config.py`

#### Track C: Hierarchy Tables ✓
**Scope**: `agents/orm/`, `agents/database/`

Completed tasks:
1. Created `Activity` ORM model with fields:
   - id, deployment_id, start_at, end_at, summary, project_id
2. Created `Project` ORM model with fields:
   - id, name, summary (intentionally no deployment_id - projects span machines)
3. Created `DaySummary` ORM model with fields:
   - id, deployment_id (nullable for cross-deployment), date, summary
4. Created PostgreSQL migration `create_hierarchy_tables.sql`
5. Updated `__init__.py` and `__all__.py` exports

New files:
- `agents/orm/activity.py`
- `agents/orm/project.py`
- `agents/orm/day_summary.py`
- `agents/database/create_hierarchy_tables.sql`

#### Track D: Secrets Infrastructure ✓
**Scope**: `agents/orm/`, `agents/database/`

Completed tasks:
1. Created `Secret` ORM model with fields:
   - id, frame_id, secret_type, raw_value (bytes), key_id, detected_at
2. Created PostgreSQL migration `create_secrets_table.sql`
3. Added to ORM exports

**Note**: Encryption is NOT yet implemented. Field is named `raw_value` (not `encrypted_value`) to reflect plaintext storage. FIDO2 implementation pending (see Issue #46).

New files:
- `agents/orm/secret.py`
- `agents/database/create_secrets_table.sql`

#### Track E: CI Pipeline ✓
**Scope**: `.github/workflows/`

Completed tasks:
1. Created `ci.yml` with:
   - Rust job: cargo fmt --check, cargo clippy, cargo build, cargo test
   - Python job: ruff check, ruff format --check, pytest
   - Docs job: verify required files exist in docs/
2. Trigger on push and PR to main
3. Use workflow_dispatch for manual runs
4. Added Rust/Python caching for faster builds
5. Added job timeouts and explicit permissions

Files modified:
- `.github/workflows/ci.yml`

---

## Remaining Work (Follow-up Issues)

### Issue #44: Rename SQLite `device_name` to `deployment_id`
**Severity**: CRITICAL - Causes query failures
**Status**: Open
**Resolved:** SQLite removed entirely on 2026-02-09. No longer applicable.

**Problem**: SQLite migrations still create `device_name` column, but Rust code queries for `deployment_id`.

| Layer | Column | File |
|-------|--------|------|
| SQLite | `device_name` | `20250131232938_add_device_name_to_frame.sql` |
| PostgreSQL | `deployment_id` | `20251119010000_schema_extensions.sql` |
| Rust queries | `deployment_id` | `db.rs`, `postgres.rs` |

**Fix Required**: Create new SQLite migration to rename column.

**Impact**: Blocks SQLite users. PostgreSQL users unaffected.

---

### Issue #45: Standardize Python DateTime Types
**Severity**: MODERATE - Type correctness
**Status**: Open

**Problem**: Some ORM models missing `DateTime(timezone=True)` for TIMESTAMPTZ columns.

**Files Affected**:
- `agents/orm/activity.py:17-18`
- `agents/orm/secret.py:28`
- `agents/orm/semantic_memory.py:94` (uses `Mapped[DateTime]` instead of `Mapped[datetime]`)

**Fix Required**: Add explicit `DateTime(timezone=True)` type specification.

**Impact**: Minor - works but less explicit about timezone handling.

---

### Issue #46: Improve Secrets Encryption Documentation
**Severity**: MODERATE - Security transparency
**Status**: Open

**Problem**: Documentation should explicitly warn that encryption is not yet implemented.

**Current State**:
- Field named `raw_value` (good - honest naming)
- Docstrings mention "FIDO2 implementation pending"

**Enhancement Required**:
- Add prominent WARNING in class docstring
- Update table-level SQL comments
- Create follow-up issue for FIDO2 implementation

**Impact**: Low - documentation improvement only.

---

### Issue #47: Future MIRIX Migration Plan
**Severity**: LOW - Technical debt
**Status**: Open

**Context**: MIRIX models (`Agent`, `Block`, `Message`, `Tool`, `File`, `Provider`, `SandboxConfig`, `CloudFileMapping`) still use `organization_id`/`user_id` from inherited multi-tenant architecture.

**Decision**: Deferred to separate project. MIRIX is experimental, and forcing migration now risks breaking working agent code.

**Required for Future**:
1. Migrate MIRIX models to `DeploymentMixin`
2. Remove `organization.py` and `user.py`
3. Update ~178 org_id/user_id references in `agents/services/`
4. Test agent functionality after migration

**Impact**: None - MIRIX subsystem remains functional with legacy schema.

---

## Issue Cross-Reference

| Issue | Severity | Track | Status | Description |
|-------|----------|-------|--------|-------------|
| #44 | CRITICAL | A | Open | SQLite `device_name` → `deployment_id` rename |
| #45 | MODERATE | B/C | Open | Python DateTime type annotations |
| #46 | MODERATE | D | Open | Secrets encryption documentation |
| #47 | LOW | B | Open | Future MIRIX migration plan |

---

## Execution Order

Tracks A, B, D, E completed in parallel. Track C completed after Track B.

```
┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐
│ Track A │  │ Track B │  │ Track D │  │ Track E │
│  Rust   │  │ Python  │  │ Secrets │  │   CI    │
│  DONE   │  │  DONE   │  │  DONE   │  │  DONE   │
└────┬────┘  └────┬────┘  └─────────┘  └─────────┘
     │            │
     │            ▼
     │       ┌─────────┐
     │       │ Track C │
     │       │Hierarchy│
     │       │  DONE   │
     │       └─────────┘
     │            │
     ▼            ▼
┌─────────────────────────────────────────────────┐
│          Polish (Issues #44-47)                 │
│          Blocked: CI billing suspension         │
└─────────────────────────────────────────────────┘
```

---

## Success Criteria

### Achieved ✓

1. `deployment_id` exists on all memory/capture tables
2. `user_id`/`organization_id` removed from memory tables (MIRIX preserved intentionally)
3. Hierarchy tables exist: activities, projects, day_summaries
4. Secrets table exists with honest naming (raw_value, not encrypted_value)
5. CI pipeline implemented with Rust/Python checks

### Remaining

6. SQLite migrations consistent with PostgreSQL (Issue #44)
7. Python DateTime types fully specified (Issue #45)
8. Secrets documentation improved (Issue #46)
9. MIRIX migration plan documented (Issue #47)

---

## Migration Safety

### Pre-Migration Requirements

1. **Set deployment ID explicitly**:
   ```bash
   psql -c "SET app.deployment_id = 'your-machine-name';" -f migrate_database_postgresql.sql
   ```

2. **Take full database backup**:
   ```bash
   pg_dump -Fc recall_db > backup_pre_migration.dump
   ```

### Rollback Procedure

If migration fails or causes issues:

1. **Transaction rollback**: Automatic (migrations wrapped in BEGIN/COMMIT)
2. **Manual restore**:
   ```bash
   pg_restore -d recall_db backup_pre_migration.dump
   ```
3. **DO NOT re-run** without fixing issues - some operations are non-idempotent

---

## Known Limitations

### MIRIX Subsystem

MIRIX models (Agent, Block, Message, etc.) retain multi-tenant schema:
- `organization_id` and `user_id` foreign keys still active
- `Organization` and `User` tables must NOT be dropped
- Full migration deferred to Issue #47

**Why**: MIRIX is experimental. Breaking working agents to achieve schema purity is not worth the risk for this PR. Pragmatic deferral is appropriate.

### Secrets Encryption

Secrets table stores **plaintext** data despite field name `raw_value`:
- No FIDO2 implementation yet
- Hardware key requirement not enforced
- Documentation warns against storing highly sensitive data

**Why**: Encryption infrastructure requires significant work (FIDO2 library integration, key management). Schema is ready, implementation follows in separate project.

### SQLite Support

SQLite migrations lag behind PostgreSQL:
- Still use `device_name` column (Issue #44)
- Hierarchy tables not yet supported in SQLite

**Why**: PostgreSQL is primary deployment target. SQLite support is development/testing convenience.

---

## Verification Checklist

### Pre-Merge (CI Blocked)

- [x] PostgreSQL migrations run without error
- [x] All `deployment_id` columns use `TEXT` type
- [x] Hierarchy tables created with foreign keys
- [x] Secrets table created with correct types
- [x] Memory tables migrated to `DeploymentMixin`
- [x] MIRIX tables preserved with org/user schema
- [x] ORM exports updated
- [x] CI pipeline implemented
- [ ] `cargo test` passes (blocked - billing)
- [ ] `pytest tests/` passes (blocked - billing)
- [ ] CI completes successfully (blocked - billing)

### Post-Billing Resolution

- [ ] CI passes on GitHub Actions
- [ ] SQLite `deployment_id` column created (Issue #44)
- [ ] Python DateTime types standardized (Issue #45)
- [ ] Secrets documentation improved (Issue #46)
- [ ] MIRIX migration plan documented (Issue #47)

---

## Next Steps

### When CI/Billing Resolves (End of Month)

1. **Run full CI pipeline** - Verify Rust/Python tests pass
2. **Address any test failures** - Fix issues surfaced by automated tests
3. **Merge to main** - Schema unification core work complete
4. **Create follow-up issues** - If #44-47 not already completed

### Optional Enhancements (Separate PRs)

- Implement FIDO2 encryption for secrets table
- Complete MIRIX migration to `deployment_id`
- Add hierarchy table support to SQLite
- Improve migration safety (better rollback, validation)

---

## References

- **Architecture**: `docs/architecture/overview.md`
- **Storage Domain**: `docs/domains/storage.md`
- **PR**: #43 (schema-unification branch)
- **Follow-up Issues**: #44 (SQLite rename), #45 (DateTime), #46 (Secrets docs), #47 (MIRIX plan)
- **Original Review**: 2025-11-27 (7 parallel review agents)

---

## Appendix: Files Modified Summary

### Rust Layer (Track A)
```
capture/screenpipe-db/src/migrations_pg/20251119000000_init.sql
capture/screenpipe-db/src/migrations_pg/20251119010000_schema_extensions.sql
capture/screenpipe-db/src/db.rs
capture/screenpipe-db/src/postgres.rs
capture/screenpipe-core/src/lib.rs
capture/screenpipe-vision/src/core.rs
capture/screenpipe-audio/src/core.rs
```

### Python Memory Layer (Track B)
```
agents/orm/mixins.py                    # Added DeploymentMixin
agents/orm/episodic_memory.py           # OrganizationMixin → DeploymentMixin
agents/orm/semantic_memory.py           # OrganizationMixin → DeploymentMixin
agents/orm/procedural_memory.py         # OrganizationMixin → DeploymentMixin
agents/orm/resource_memory.py           # OrganizationMixin → DeploymentMixin
agents/orm/knowledge_vault.py           # OrganizationMixin → DeploymentMixin
agents/orm/__init__.py                  # Updated exports
agents/orm/__all__.py                   # Updated exports
agents/schemas/*.py                     # Updated schemas
agents/database/migrate_database_postgresql.sql  # Migration script
```

### Hierarchy Layer (Track C)
```
agents/orm/activity.py                  # NEW
agents/orm/project.py                   # NEW
agents/orm/day_summary.py               # NEW
agents/database/create_hierarchy_tables.sql  # NEW
```

### Secrets Layer (Track D)
```
agents/orm/secret.py                    # NEW
agents/database/create_secrets_table.sql     # NEW
```

### CI Layer (Track E)
```
.github/workflows/ci.yml                # NEW
```

### Preserved (MIRIX)
```
agents/orm/organization.py              # KEPT (legacy)
agents/orm/user.py                      # KEPT (legacy)
agents/orm/agent.py                     # KEPT (uses org/user)
agents/orm/block.py                     # KEPT (uses org/user)
agents/orm/message.py                   # KEPT (uses org/user)
agents/orm/tool.py                      # KEPT (uses org/user)
agents/orm/file.py                      # KEPT (uses org/user)
agents/orm/provider.py                  # KEPT (uses org/user)
agents/orm/sandbox_config.py            # KEPT (uses org/user)
```
