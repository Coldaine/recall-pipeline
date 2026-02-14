# Testing Philosophy for Recall Pipeline

**Status:** Production codebase with no tests. This document establishes what tests actually matter.

---

## The Core Truth: Unit Tests Are Mostly Pointless

### What Unit Tests Actually Test

A unit test verifies: **"When I call this function with these inputs, it returns this output."**

```python
def load_image(path: str) -> Image:
    path = Path(path)
    if not path.exists():
        return None
    return Image.open(path)

# Unit test
def test_load_image_missing_file():
    result = load_image("/nonexistent/path.png")
    assert result is None
```

**What this test verifies:**
- PIL.Image.open() works (already tested by PIL)
- Path.exists() works (already tested by Python stdlib)
- Our if-statement executes correctly (trivial logic)

**What it doesn't verify:**
- Whether real files can be loaded in production
- Whether PIL handles corrupted images gracefully
- Whether the path is correct at runtime
- Whether the image dimensions are sane
- Whether concurrent access causes issues

---

## Why This Doesn't Matter

### 1. Libraries Are Already Tested

**PIL (Python Imaging Library):**
- Image.open() has been tested by Pillow developers on millions of images
- Path.exists() has been tested by CPython developers for decades
- If PIL is broken, it's broken in production too — a unit test won't save you

**Tesseract OCR:**
- pytesseract has tests
- Tesseract has tests
- If Tesseract fails on an image type, we'll find out when we run it on real images

**PostgreSQL:**
- Extensively tested
- If our INSERT fails, it fails the same way in integration tests
- We don't need a unit test to verify conn.execute() works

### 2. Mocking Hides Real Problems

```python
# Unit test with mock
@patch('PIL.Image.open')
def test_load_image(mock_open):
    mock_open.return_value = MagicMock()
    result = load_image("test.png")
    assert result is not None  # ✓ PASS

# But in production:
image = load_image("test.png")  # ✗ FAIL: Corrupted image, PIL throws exception
```

The mock test passed. The code failed.

### 3. You Catch Problems Wider Tests

When you run an **integration test:**

```python
async def test_ocr_worker_processes_real_image():
    """Real test: Load image, run Tesseract, verify text extracted."""
    worker = OCRWorker()
    
    # Real image file on disk
    image = worker.load_image("test_images/clear_text.png")
    assert image is not None  # ← Catches image loading bugs
    
    # Real Tesseract run
    text, confidence = worker.run_ocr(image)
    assert text == "EXPECTED_TEXT"  # ← Catches OCR bugs
    assert 0 <= confidence <= 100  # ← Catches confidence calculation bugs
```

This test:
- ✅ Validates image loading works
- ✅ Validates OCR extraction works
- ✅ Validates confidence calculation is correct
- ✅ Validates text matches expectations

All the same things the mocked unit tests would check, **plus** the things that actually matter.

---

## What Tests Actually Matter

### 1. Integration Tests (Real behavior validation)

**These are worth writing:**

```python
# Real file I/O, no mocks
async def test_ocr_load_image_absolute_path():
    """Can we load a real PNG file with absolute path?"""
    worker = OCRWorker()
    image = worker.load_image("/tmp/test_image.png")
    assert image is not None
    assert image.size[0] > 0
    assert image.size[1] > 0

# Real Tesseract, no mocks
async def test_ocr_real_tesseract_clear_text():
    """Does Tesseract actually extract text from a real image?"""
    worker = OCRWorker()
    image = worker.load_image("test_images/clear_text.png")
    text, confidence = worker.run_ocr(image)
    assert "expected text" in text.lower()
    assert confidence > 0.7

# Real database, no mocks
async def test_ocr_worker_updates_database():
    """Does OCRWorker actually update the database?"""
    db = get_test_db()
    
    # Insert a test frame
    frame_id = await db.insert_frame(
        id=uuid4(),
        image_ref="test_image.png",
        # ...
    )
    
    # Run OCR worker
    worker = OCRWorker()
    result = await worker.process_frame(frame_from_db)
    
    # Verify database was updated
    updated = await db.get_frame(frame_id)
    assert updated.ocr_text is not None
    assert updated.vision_status == 2
```

These tests:
- ✅ Use real libraries (PIL, Tesseract, asyncpg)
- ✅ Use real I/O (disk files, database)
- ✅ Catch real failures (missing file, Tesseract crash, DB error)
- ✅ Validate actual behavior (not mock behavior)

### 2. End-to-End Tests (Full pipeline)

```python
async def test_full_ocr_vision_pipeline():
    """Real test: Frame through entire pipeline."""
    db = get_test_db()
    
    # Insert real frame with real image
    frame_id = await db.insert_frame(
        image_ref="test_images/screenshot.png",
        vision_status=0  # pending
    )
    
    # Run OCR worker to completion
    ocr_worker = OCRWorker(poll_interval=0.1)
    await ocr_worker.process_batch(db)
    
    # Verify status changed
    frame = await db.get_frame(frame_id)
    assert frame.vision_status == 2  # OCR done
    assert frame.ocr_text is not None
    
    # Run Vision worker to completion
    vision_worker = VisionWorker(poll_interval=0.1)
    await vision_worker.process_batch(db)
    
    # Verify final status
    frame = await db.get_frame(frame_id)
    assert frame.vision_status == 4  # vision done
    assert frame.vision_summary is not None
```

This test validates:
- ✅ OCR pipeline actually extracts text
- ✅ Vision pipeline actually generates summaries
- ✅ Both workers can run sequentially
- ✅ Database state transitions correctly
- ✅ No data is lost or corrupted

### 3. Error Handling Tests (Real failure modes)

```python
async def test_ocr_missing_image_at_runtime():
    """What happens if image disappears between fetch and process?"""
    db = get_test_db()
    
    # Insert frame with image that exists
    frame_id = await db.insert_frame(image_ref="test_image.png")
    
    # Now delete the image (simulates runtime race condition)
    os.remove("test_image.png")
    
    # Worker should handle gracefully
    worker = OCRWorker()
    result = await worker.process_frame(frame_from_db)
    
    assert result.error is not None
    assert "not found" in result.error.lower()

async def test_vision_api_key_invalid():
    """What happens if API key is invalid?"""
    worker = VisionWorker(model="gpt-4o")
    # Set invalid API key
    os.environ["OPENAI_API_KEY"] = "sk-invalid"
    
    # Should fail gracefully, not hang
    result = await worker.process_frame(frame)
    assert result.error is not None
    assert "authentication" in result.error.lower() or "unauthorized" in result.error.lower()

async def test_database_connection_recovery():
    """Does worker recover if database connection drops?"""
    db = get_test_db()
    worker = OCRWorker(max_retries=3, retry_delay=0.1)
    
    # Simulate connection drop by terminating backend
    await db.pool().execute("SELECT pg_terminate_backend(pid) FROM pg_stat_activity")
    
    # Worker should retry and eventually succeed (new connections created)
    processed = await worker.run_with_retry(db)
    assert processed >= 0  # No crash
```

These tests:
- ✅ Validate error handling actually works
- ✅ Test real failure scenarios (not mocked failures)
- ✅ Ensure code doesn't crash or hang
- ✅ Verify recovery logic is sound

### 4. Performance Tests (Actual throughput)

```python
async def test_ocr_throughput_with_real_images():
    """Process 100 real images, measure throughput."""
    db = get_test_db()
    worker = OCRWorker(batch_size=10)
    
    # Insert 100 real test images
    for i in range(100):
        await db.insert_frame(
            image_ref=f"test_images/screenshot_{i}.png",
            vision_status=0
        )
    
    # Measure processing time
    start = time.time()
    total_processed = 0
    while True:
        processed = await worker.process_batch(db)
        total_processed += processed
        if total_processed >= 100:
            break
    
    elapsed = time.time() - start
    throughput = total_processed / elapsed
    
    # Verify minimum throughput
    assert throughput > 5  # At least 5 frames/sec
    print(f"OCR throughput: {throughput:.1f} frames/sec")
```

This validates:
- ✅ Code actually completes (not hanging)
- ✅ Performance meets minimum requirements
- ✅ No memory leaks (no exponential slowdown)

---

## What NOT to Test

### ❌ Mocked Unit Tests

```python
# DON'T WRITE THIS
@patch('agents.processors.ocr_worker.OCRWorker.load_image')
def test_process_frame_with_mock(mock_load):
    """This tests the mock, not the code."""
    mock_load.return_value = MagicMock()
    # ...
```

**Why:** If load_image() is actually broken, this test passes anyway.

### ❌ Tests That Duplicate Integration Tests

```python
# DON'T WRITE THIS if integration test exists
def test_fetch_pending_frames_returns_list():
    """Unit test: Query returns correct structure."""
    # Mock database
    # ...

# This is redundant with:
async def test_fetch_pending_frames_from_real_db():
    """Integration test: Query returns correct structure from real DB."""
    # Real database
    # ...
```

**Why:** If the integration test passes, the unit test adds nothing.

### ❌ Tests for Library Functions

```python
# DON'T WRITE THIS
def test_image_open():
    """Verify PIL.Image.open() works."""
    image = Image.open("test.png")
    assert image is not None
```

**Why:** PIL tests this. Write integration tests that use Image.open() as part of larger operations.

### ❌ Tests That Test Mock Framework

```python
# DON'T WRITE THIS
def test_mock_was_called_with_correct_args():
    """Verify mock.assert_called_with() works."""
    # ...
```

**Why:** You're testing unittest.mock, not your code.

---

## Test Organization

### Real Tests We Actually Need

#### Tesseract OCR (No infrastructure needed)
```
tests/
  test_ocr_image_loading.py
    - test_load_png_absolute_path()
    - test_load_jpeg_relative_path()
    - test_load_corrupted_image()
    - test_load_missing_file()
  
  test_ocr_tesseract.py
    - test_tesseract_clear_text()
    - test_tesseract_blurry_text()
    - test_tesseract_rotated_text()
    - test_confidence_range()
```

#### Database Operations (Needs test PostgreSQL)
```
tests/integration/
  test_database_frames.py
    - test_insert_frame()
    - test_fetch_pending_frames()
    - test_mark_frames_processing()
    - test_concurrent_frame_processing()
  
  test_database_ocr_text.py
    - test_insert_ocr_text()
    - test_ocr_text_persistence()
```

#### Full Pipeline (Needs test infrastructure)
```
tests/e2e/
  test_ocr_pipeline.py
    - test_ocr_worker_full_cycle()
    - test_concurrent_ocr_workers()
  
  test_vision_pipeline.py
    - test_vision_worker_full_cycle()
    - test_vision_rate_limiting()
  
  test_full_pipeline.py
    - test_frame_through_entire_pipeline()
    - test_concurrent_ocr_and_vision()
```

#### Error Handling (Needs error injection)
```
tests/e2e/
  test_error_recovery.py
    - test_missing_image_at_runtime()
    - test_database_connection_lost()
    - test_invalid_api_key()
    - test_worker_graceful_shutdown()
```

#### Performance (Needs real workload)
```
tests/performance/
  test_throughput.py
    - test_ocr_throughput()
    - test_vision_throughput()
    - test_concurrent_throughput()
  
  test_memory.py
    - test_memory_usage_large_batch()
```

---

## Test Count Reality

**Forget the 55 tests I recommended.**

**What's actually needed:**

| Category | Count | Infrastructure |
|----------|-------|-----------------|
| Image Loading | 5 | None (disk files) |
| Tesseract OCR | 5 | Tesseract only |
| Database Ops | 10 | Test PostgreSQL |
| Polling Loop | 5 | Test PostgreSQL |
| Error Handling | 8 | Test PostgreSQL + error injection |
| E2E Pipeline | 5 | Full infrastructure |
| Performance | 3 | Full infrastructure + measurement |
| **Total** | **~40** | **Varies** |

**Not 55, not 100. 40 tests that matter.**

---

## Implementation Priority

### Phase 1: Image & OCR (Can start TODAY)
```bash
# No external dependencies beyond what's already installed
tests/test_ocr_image_loading.py       # ~5 tests
tests/test_ocr_tesseract.py           # ~5 tests
```

**Why first:** No infrastructure needed, validates core OCR works.

### Phase 2: Database Setup (Needs test PostgreSQL)
```bash
# Set up test PostgreSQL container
tests/integration/test_database_frames.py        # ~5 tests
tests/integration/test_database_ocr_text.py      # ~5 tests
```

**Why second:** Builds on Phase 1, enables integration tests.

### Phase 3: Full Pipeline (Needs all infrastructure)
```bash
# All systems go
tests/e2e/test_ocr_pipeline.py
tests/e2e/test_vision_pipeline.py
tests/e2e/test_error_recovery.py
tests/performance/test_throughput.py
```

**Why last:** Depends on everything working.

---

## The Rule

### ✅ Write a test if:
- It touches real I/O (disk, database, network)
- It validates a workflow works end-to-end
- It catches a real failure mode
- It measures a performance requirement

### ❌ Skip a test if:
- It mocks a function in your own code
- It tests a library function (PIL, asyncpg, Tesseract already test their own code)
- It duplicates what an integration test already validates
- It tests the test framework itself

---

## Summary

**Stop writing mocked unit tests.**

They give false confidence. They pass when the code is broken. They duplicate what integration tests already validate.

**Write real tests.**

- Real images
- Real Tesseract
- Real database
- Real API errors
- Real concurrency

**Test count:**
- ❌ 55 mocked + speculative tests
- ✅ 40 real tests that validate actual behavior

**First step:** Write the 10 tests from Phase 1 (image loading + OCR). Everything else builds from there.

