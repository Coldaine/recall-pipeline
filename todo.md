---
last_updated: 2026-02-17
status: active
---

# Recall Pipeline - Task Backlog

## 🏗️ Architecture & Configuration (CRITICAL)

### [CONFIG-001] Multi-Monitor Configuration System
**Status**: Not Started  
**Priority**: P0 (Blocks per-monitor tuning)  
**Epic**: Configuration Management  

Replace CLI args with config file (TOML/YAML) to support per-monitor settings:
- Monitor selection (enabled flag, name/id matching)
- Per-monitor FPS (portrait monitor vs ultrawide)
- Per-monitor dedup threshold
- Per-monitor max_inactive_secs (force capture even if deduped)

**Files to update**:
- `capture/src/bin/recall.rs` (Args struct, main loop)
- `capture/recall-capture/src/pipeline.rs` (run_capture_task)

**Config format** (example):
```toml
[monitors]
[[monitors.specs]]
name = "VX2768-2KP"  # Portrait
fps = 1.0
dedup_threshold = 0.01
max_inactive_secs = 60
enabled = true

[[monitors.specs]]
name = "Odyssey G8"  # Ultrawide
fps = 0.5
dedup_threshold = 0.005
max_inactive_secs = 30
enabled = true
```

**Definition of Done**:
- Config file loader (serde + toml crate)
- Monitor matching by name/id
- Per-monitor config passed to capture tasks
- Config file path: `~/.config/recall.toml` (default)
- Graceful fallback to CLI args if no config file

---

### [CONFIG-002] Hardcoded Dedup Threshold
**Status**: Not Started  
**Priority**: P1 (Blocks per-monitor tuning)  
**Depends on**: CONFIG-001  

Make `DEDUP_THRESHOLD` configurable per-monitor (currently hardcoded at 0.006).

**Files**:
- `capture/recall-capture/src/pipeline.rs` (line 158)

**Why**: Portrait monitor may need aggressive dedup (0.01), ultrawide may need sensitive dedup (0.005).

---

## 🐛 Bugs & Data Integrity

### [BUG-001] Capture Timestamp Not Preserved
**Status**: Not Started  
**Priority**: P0 (Data integrity issue)  
**Epic**: Timestamp Accuracy  

**Problem**: 
- Frame captured at T=10:00:00 but stored at T=10:02:34 (if storage task is slow)
- Database `captured_at` uses `Utc::now()` instead of actual capture time
- Makes timeline reconstruction impossible

**Files**:
- `capture/src/bin/recall.rs` (lines 270, 356)
- `capture/recall-capture/src/pipeline.rs` (CaptureMessage struct)

**Fix**:
1. Store `DateTime<Utc>` instead of `Instant` in CaptureMessage
2. Capture timestamp immediately after monitor.capture_image()
3. Pass through dedup → storage pipeline unchanged

**Definition of Done**:
- Unit test verifies timestamp is preserved end-to-end
- Timestamp matches within 100ms of actual capture

---

## 📊 Monitoring & Observability

### [METRICS-001] Per-Monitor Metrics
**Status**: Not Started  
**Priority**: P2 (Observability)  
**Depends on**: CONFIG-001  

Currently metrics are global only:
```
frames_captured: 500        # No breakdown by monitor
frames_deduped_memory: 400
frames_stored: 100
```

Need per-monitor metrics:
```
monitor_0 (VX2768-2KP):
  - captured: 250
  - deduped_memory: 200
  - stored: 50

monitor_1 (Odyssey G8):
  - captured: 250
  - deduped_memory: 200
  - stored: 50
```

**Files**:
- `capture/recall-capture/src/pipeline.rs` (PipelineMetrics struct)
- `capture/src/bin/recall.rs` (metrics logging)

**Definition of Done**:
- Per-monitor counters in PipelineMetrics
- Log per-monitor breakdown on shutdown
- Export per-monitor metrics in final summary

---

## 🧪 Testing & Quality

### [TEST-001] Restructure Test Suite
**Status**: In Progress (created pipeline_integration.rs)  
**Priority**: P1 (Blocks CI confidence)  
**Epic**: Test Infrastructure  

Currently tests are:
- Scattered (inline + integration)
- No Python tests (placeholder only)
- No CI config
- No hardware-skip strategy for CI

**Tasks**:
1. [TEST-001-A] Create test directory structure
   - `capture/recall-capture/tests/` (move integration tests here)
   - `agents/tests/` (create Python tests)
   - `tests/integration/` (multi-crate tests)

2. [TEST-001-B] Add `just test` command
   - Run Rust: `cargo test --workspace`
   - Run Python: `pytest tests/`
   - Run all: `cargo test --workspace && pytest tests/`

3. [TEST-001-C] Add hardware test handling
   - Skip tests that need monitors if `SKIP_HARDWARE=1`
   - Mark with `#[ignore]` if no display available

4. [TEST-001-D] Write Python agent tests
   - Test episodic_memory_agent
   - Test semantic_memory_agent
   - Test ORM models

**Files**:
- `justfile` (add test targets)
- `capture/recall-capture/tests/pipeline_integration.rs` (already done ✓)
- `tests/README.md` (document test strategy)

**Definition of Done**:
- `just test` passes locally
- Tests run in headless CI
- >70% code coverage reported

---

### [TEST-002] Multi-Monitor Integration Test
**Status**: Done ✓  
**Priority**: P2  

Created `test_live_hardware_capture()` that:
- Detects all monitors
- Captures screenshot from each
- Verifies RGBA8 format and dimensions

**Test file**: `capture/recall-capture/tests/pipeline_integration.rs`

---

## 📚 Documentation

### [DOC-001] Test Strategy Guide
**Status**: Not Started  
**Priority**: P2  

Create `tests/README.md` documenting:
- Where tests go (directories)
- How to run tests (`just test`)
- Hardware test handling
- Adding new tests
- CI/CD expectations

---

## 🚀 Performance & Optimization

### [PERF-001] Max Inactive Seconds (Force Capture)
**Status**: Not Started  
**Priority**: P2 (QoL)  
**Depends on**: CONFIG-001  

Currently: If screen doesn't change for 10 minutes, no capture sent.  
Need: Option to force capture every N seconds (e.g., 60) even if deduped.

**Files**:
- `capture/recall-capture/src/pipeline.rs` (run_capture_task)

**Implementation**:
```rust
let max_inactive = Duration::from_secs(config.max_inactive_secs);
let mut last_forced_capture = Instant::now();

if last_forced_capture.elapsed() >= max_inactive {
    // Send frame even if deduped
    last_forced_capture = Instant::now();
}
```

---

## 📋 Summary

| Tag | Count | Status |
|-----|-------|--------|
| CONFIG | 2 | 🚫 Not Started |
| BUG | 1 | 🚫 Not Started |
| METRICS | 1 | 🚫 Not Started |
| TEST | 2 | 🔄 In Progress (1 done) |
| DOC | 1 | 🚫 Not Started |
| PERF | 1 | 🚫 Not Started |

**Critical Path**:
1. CONFIG-001 (config file system)
2. BUG-001 (timestamp preservation)
3. TEST-001 (test suite restructure)

All other tasks unblock after these three.
