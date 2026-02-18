# Testing Strategy

> **Philosophy**: We rely on a **Local-First Tiered Testing** strategy. Because cloud CI can be flaky or blocked, developers must verify quality *locally* before pushing.

## 🏆 The Three Tiers

All code changes must pass these gates.

### 🟢 Tier 1: Syntax & Style (Fast)
**When:** Pre-commit / On-save.
**Time Budget:** < 5 seconds.
**Command:**
```bash
just lint
# Runs: cargo fmt --check && cargo clippy -- -D warnings
```

### 🟡 Tier 2: Logic & Units (Medium)
**When:** Pre-push.
**Time Budget:** < 1 minute.
**Command:**
```bash
just test-unit
# Runs: cargo test --lib --workspace
```
**Scope:**
- Pure functions (math, parsing, dedup logic).
- Mocks for DB/Network.
- **NO** requires running services (Postgres).

### 🔴 Tier 3: Integration & System (Slow)
**When:** Before opening a PR / Pre-Merge.
**Time Budget:** < 5 minutes.
**Command:**
```bash
just test-int
# Runs: cargo test --test '*' --workspace
```
**Requirements:**
- Local Docker container with Postgres + pgvector + TimescaleDB.
- `DATABASE_URL` set in `.env`.
- Validates full pipeline: Capture -> DB -> Query.

---

## 🐛 Debugging Guide

### "The Timestamp Bug" Regression Test
Ensure timestamps are preserved:
```bash
cargo test --package recall-capture --test pipeline_integrity
```

### Mocking Strategies
- **Database:** Use `sqlx::test` for integration tests. Use traits/mocks for unit tests.
- **Time:** Never call `Instant::now()` directly in logic. Accept time as an argument or use a `Clock` trait.
- **Filesystem:** Use `tempfile` crate for file I/O tests.

---

## 📈 Performance Benchmarks

Run benchmarks to ensure we stay within budget:
```bash
cargo bench
```

**Key Metrics to Watch:**
- Hash calculation time (target: < 1ms)
- Dedup comparison time (target: < 0.1ms)
- DB Insert throughput (target: > 1000 rows/sec)
