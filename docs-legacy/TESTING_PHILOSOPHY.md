# Testing Philosophy

## The Core Principle

**Write tests that use real dependencies; skip tests that mock your own code.**

A unit test verifies: "When I call this function with these inputs, it returns this output."

A mocked test verifies: "When I call this function, it calls the mock I set up."

Neither verifies: "The code actually works in production."

**Libraries are already tested.** PIL, Tesseract, PostgreSQL, asyncpg - these have extensive test suites. Your mock doesn't add coverage. If PIL is broken, it's broken in production too - a mocked test won't catch it.

**Real bugs happen at integration points.** The file doesn't exist. The image is corrupted. The database connection drops. Mocks hide these failures.

---

## Why Mocks Are Not Helpful

A mocked test verifies that your code calls a function. It doesn't verify the function works.

```python
# DON'T: Mock your own code
@patch('PIL.Image.open')
def test_load_image(mock_open):
    mock_open.return_value = MagicMock()
    result = load_image("test.png")
    assert result is not None  # Passes, but proves nothing
```

**The problem:** If `Image.open()` throws on corrupted images, you won't know until production. The mock test passed. The code failed.

**Libraries are already tested.** PIL, Tesseract, PostgreSQL - these have extensive test suites. If they're broken, your mocked test won't catch it.

---

## Why Unit Tests Are Not Helpful (Usually)

A unit test typically verifies: "When I call this function with X, it returns Y."

```python
# DON'T: Test library code
def test_path_exists():
    path = Path("/tmp")
    assert path.exists()  # Tests stdlib, not your code
```

**The problem:** You're testing Python's `Path.exists()`, not your code. The logic is trivial. The library is already tested.

**What unit tests miss:**
- Whether real files can be loaded
- Whether PIL handles corrupted images
- Whether concurrent access causes issues
- Whether the path is correct at runtime

---

## What Actually Works: Integration Tests

### What Is an Integration Test?

An integration test verifies that multiple components work together correctly. Unlike a unit test (which isolates one function), an integration test uses real dependencies:

- Real files on disk
- Real database connections
- Real external processes (Tesseract, Whisper)
- Real hardware (screens, audio devices)

### How to Scope an Integration Test

**Test one workflow, not the whole system.**

```python
# GOOD: One workflow (OCR extraction)
async def test_ocr_extracts_text():
    worker = OCRWorker()
    image = worker.load_image("tests/fixtures/screenshot.png")
    text, confidence = worker.run_ocr(image)
    assert "expected text" in text.lower()

# BAD: Entire pipeline (too broad)
async def test_everything():
    await capture_screen()
    await run_ocr()
    await run_vision()
    await save_to_db()
    await notify_user()  # Too many things to fail
```

**Why scope matters:**
- Narrow tests pinpoint what broke
- Broad tests are brittle (many failure points)
- Each test should verify one behavior

### Integration Test Example

Test with real dependencies. One test catches what ten mocked tests miss.

```python
# DO: Real image, real Tesseract
async def test_ocr_extracts_text():
    worker = OCRWorker()
    
    image = worker.load_image("tests/fixtures/screenshot.png")
    text, confidence = worker.run_ocr(image)
    
    assert "expected text" in text.lower()
    assert confidence > 0.7
```

This catches:
- Image loading bugs
- Tesseract failures  
- Confidence calculation bugs
- Text extraction bugs

All in one test.

### Live Environments are Default

We run tests in **live environments** (e.g., dev workstation with monitors).

- **No Skips**: Tests should fail if hardware/dependencies are missing, not skip.
- **No Fallbacks**: Do not fallback to mocks if a service is down. Fail the test.
- **CI is Special**: Only disable hardware tests if explicitly flagged (e.g., `SKIP_HARDWARE=1`).

This ensures we are testing the actual code path that runs in production. No mocks needed.

---

## Error Handling: Test Real Conditions

Use inputs that naturally cause errors - no mocking needed.

```python
# DO: Reference a file that doesn't exist
async def test_ocr_handles_missing_file():
    worker = OCRWorker()
    
    # Use a path that doesn't exist
    result = await worker.process_frame(
        Frame(image_ref="tests/fixtures/nonexistent.png")
    )
    
    assert result.error is not None
    assert "not found" in result.error.lower()
```

This tests your error handling with a real FileNotFoundError from the OS.

---

## The Exceptions

### When Unit Tests ARE Worth It

**1. Pure business logic with no I/O:**

```python
# DO: Pure function worth testing
def calculate_similarity(vec1: list[float], vec2: list[float]) -> float:
    dot = sum(a * b for a, b in zip(vec1, vec2))
    norm1 = sum(a * a for a in vec1) ** 0.5
    norm2 = sum(b * b for b in vec2) ** 0.5
    return dot / (norm1 * norm2)

def test_similarity_identical_vectors():
    vec = [1.0, 2.0, 3.0]
    assert calculate_similarity(vec, vec) == 1.0
```

This is worth testing because:
- No I/O dependencies
- Deterministic output
- Non-trivial logic
- Easy to get wrong

Examples from this codebase: embedding calculations in `agents/embeddings.py`, deduplication logic in `capture/recall-core/`.

**2. Testing that a capability exists:**

Some tests verify that a capability actually works - this is not a unit test in the traditional sense, but it tests a single component:

```python
# DO: Verify screenshot capture actually works
async def test_screenshot_capture():
    capture = ScreenCapture()
    
    # Take a real screenshot
    image = capture.capture_screen()
    
    assert image is not None
    assert image.size[0] > 0
    assert image.size[1] > 0
```

This is valuable because:
- Verifies the hardware/software capability exists
- Catches driver issues, permission problems, missing displays
- Can be embedded in a larger pipeline test as well

### When Mocks ARE Worth It

External APIs that are rate-limited or expensive:

```python
# DO: Mock expensive external API
@pytest.fixture
def mock_openai():
    with patch('openai.ChatCompletion.create') as mock:
        mock.return_value = MagicMock(
            choices=[MagicMock(message=MagicMock(content="Summary"))]
        )
        yield mock

async def test_summarization(mock_openai):
    worker = VisionWorker()
    result = await worker.summarize("content")
    assert result is not None
```

This is worth mocking because:
- Costs money per call
- Rate limited
- Not your code to test

---

## Anti-Pattern: Reconstructing the Pipeline in Test Code

**The problem:** Writing an "integration test" that doesn't actually invoke your pipeline code, but instead reconstructs the pipeline logic in the test file.

```python
# DON'T: Reconstruct the pipeline in test code
async def test_ocr_pipeline_bad():
    # This is NOT testing the actual pipeline!
    db = await create_test_database()
    image = await load_test_image("test.png")
    text = await run_tesseract(image)
    await save_to_database(db, text)
    
    # You just rewrote the pipeline in test code.
    # If the real pipeline changes, this test still passes.
```

**Why this is wrong:**
- The test doesn't use the actual pipeline code
- If the pipeline implementation changes, the test won't catch it
- You're testing your test code, not your production code

**DO: Invoke the actual pipeline:**

```python
# DO: Test the actual pipeline
async def test_ocr_pipeline_good():
    # Configure pipeline for test
    pipeline = OCRPipeline(
        db_url="postgresql://test:test@localhost/test_db",
        input_dir="tests/fixtures/images"
    )
    
    # Run the ACTUAL pipeline code
    result = await pipeline.run_once()
    
    # Verify the pipeline did its job
    assert result.frames_processed > 0
    assert result.errors == []
```

This tests the real code path that runs in production. The pipeline implementation can change, and the test will still verify correct behavior.

---

## Summary

| Write | Skip |
|-------|------|
| Integration tests with real dependencies | Mocked tests of your own code |
| Unit tests for pure business logic | Unit tests for library functions |
| Tests that verify capabilities exist | Tests that reconstruct pipeline logic |
| Mocks for external APIs | Mocks for internal dependencies |

**Run tests:** `just test`