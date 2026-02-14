"""
Unit tests for OCRWorker - MOCKED TESTS ONLY.

WARNING: These tests are unit-level mocks and do NOT validate:
  - Real Tesseract OCR functionality
  - Actual image file I/O
  - Database persistence
  - The async polling loop
  - Batch processing with real database transactions
  - Error handling with actual database errors

See: tests/integration/test_ocr_pipeline.py for integration tests
     tests/e2e/test_full_pipeline.py for end-to-end tests
"""

import pytest
from unittest.mock import MagicMock, patch, AsyncMock
from uuid import uuid4
from datetime import datetime
from agents.processors.ocr_worker import OCRWorker, FrameRecord, OCRResult

@pytest.fixture
def ocr_worker():
    return OCRWorker(batch_size=1, poll_interval=0.1)

@pytest.fixture
def mock_frame():
    return FrameRecord(
        id=uuid4(),
        captured_at=datetime.now(),
        image_ref="test_image.png",
        vision_status=0
    )

@pytest.mark.asyncio
async def test_process_frame_success(ocr_worker, mock_frame):
    """
    UNIT TEST: Verify OCRResult structure when image loads and OCR succeeds.
    
    LIMITATION: Uses mocks for load_image() and run_ocr().
    This does NOT test:
      - Real image loading from disk
      - Real Tesseract OCR execution
      - Confidence value accuracy
      - Text extraction quality
    
    TODO: Add integration test that:
      1. Creates a real test image file (e.g., PNG with text)
      2. Calls OCRWorker.load_image() without mocking
      3. Calls OCRWorker.run_ocr() without mocking
      4. Verifies extracted text matches expected content
      5. Validates confidence scores are in valid range [0-100]
    """
    # Setup
    with patch('agents.processors.ocr_worker.OCRWorker.load_image') as mock_load, \
         patch('agents.processors.ocr_worker.OCRWorker.run_ocr') as mock_ocr:
        
        mock_load.return_value = MagicMock() # PIL Image
        mock_ocr.return_value = ("Extracted Text", 0.95)
        
        # Execute
        result = await ocr_worker.process_frame(mock_frame)
        
        # Verify
        assert isinstance(result, OCRResult)
        assert result.text == "Extracted Text"
        assert result.confidence == 0.95
        assert result.error is None
        assert result.frame_id == mock_frame.id

@pytest.mark.asyncio
async def test_process_frame_image_load_failure(ocr_worker, mock_frame):
    """
    UNIT TEST: Verify error handling when image file cannot be loaded.
    
    LIMITATION: Mocks the load_image() failure; doesn't test real file I/O.
    This does NOT test:
      - Missing file paths (file not found on disk)
      - Corrupted image files
      - Invalid image formats
      - Insufficient permissions
      - Path resolution with relative/absolute paths
    
    TODO: Add integration test that:
      1. Attempts to load image from non-existent path
      2. Attempts to load corrupted/invalid image file
      3. Attempts to load from path without read permissions
      4. Verifies error messages are informative and logged
      5. Verifies frame status is NOT updated in database on failure
    """
    # Setup
    with patch('agents.processors.ocr_worker.OCRWorker.load_image') as mock_load:
        mock_load.return_value = None
        
        # Execute
        result = await ocr_worker.process_frame(mock_frame)
        
        # Verify
        assert result.text == ""
        assert "Could not load image" in result.error

