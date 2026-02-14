import pytest
from unittest.mock import MagicMock, patch, AsyncMock
from uuid import uuid4
from datetime import datetime
from agents.processors.vision_worker import VisionWorker, VisionFrameRecord, VisionResult
from agents.schemas.message import Message

@pytest.fixture
def vision_worker():
    return VisionWorker()

@pytest.fixture
def mock_frame():
    return VisionFrameRecord(
        id=uuid4(),
        captured_at=datetime.now(),
        image_ref="screen.png",
        ocr_text="Detected text",
        vision_status=2
    )

@pytest.mark.asyncio
async def test_process_frame_success(vision_worker, mock_frame):
    # Setup
    with patch('agents.processors.vision_worker.VisionWorker.load_image_base64') as mock_load, \
         patch('agents.processors.vision_worker.VisionWorker._get_llm_client') as mock_get_client:
        
        mock_load.return_value = ("data:image/png;base64,123", "image/png")
        
        # Mock LLM Client behavior
        mock_client = MagicMock()
        mock_response = MagicMock()
        mock_response.choices[0].message.content = "Summary of screen."
        mock_client.send_llm_request.return_value = mock_response
        mock_get_client.return_value = mock_client

        # Execute
        result = await vision_worker.process_frame(mock_frame)
        
        # Verify
        assert isinstance(result, VisionResult)
        assert result.summary == "Summary of screen."
        assert result.error is None
        mock_client.send_llm_request.assert_called_once()

@pytest.mark.asyncio
async def test_process_frame_llm_failure(vision_worker, mock_frame):
    # Setup
    with patch('agents.processors.vision_worker.VisionWorker.load_image_base64') as mock_load, \
         patch('agents.processors.vision_worker.VisionWorker._get_llm_client') as mock_get_client:
        
        mock_load.return_value = ("data:image/png;base64,123", "image/png")
        
        mock_client = MagicMock()
        mock_client.send_llm_request.side_effect = Exception("LLM Error")
        mock_get_client.return_value = mock_client
        
        # Execute
        result = await vision_worker.process_frame(mock_frame)
        
        # Verify
        assert result.summary is None
        assert "LLM Error" in result.error
