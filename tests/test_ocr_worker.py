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
    # Setup
    with patch('agents.processors.ocr_worker.OCRWorker.load_image') as mock_load:
        mock_load.return_value = None
        
        # Execute
        result = await ocr_worker.process_frame(mock_frame)
        
        # Verify
        assert result.text == ""
        assert "Could not load image" in result.error

