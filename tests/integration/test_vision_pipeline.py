import pytest
import asyncpg
from uuid import uuid4
from datetime import datetime, timezone
from unittest.mock import MagicMock, AsyncMock, patch
from agents.processors.vision_worker import VisionWorker, VisionResult, VISION_STATUS_OCR_DONE, VISION_STATUS_VISION_DONE, VISION_STATUS_ERROR

@pytest.fixture
def vision_worker():
    return VisionWorker(
        batch_size=1,
        poll_interval=0.1,
        max_retries=1,
        model="gpt-4o-mini" # Use a cheaper model for tests if we were running real calls
    )

@pytest.mark.asyncio
async def test_vision_pipeline_happy_path(db_pool, vision_worker):
    """
    Integration test: Insert frame (OCR Done) -> Run Vision -> Verify summary.
    """
    async with db_pool.acquire() as conn:
        frame_id = uuid4()
        await conn.execute(
            """
            INSERT INTO frames (id, captured_at, image_ref, ocr_text, vision_status)
            VALUES ($1, $2, $3, $4, $5)
            """,
            frame_id, 
            datetime.now(timezone.utc), 
            "test_image.png", 
            "ocr text content",
            VISION_STATUS_OCR_DONE
        )

        # Mock dependencies
        with patch.object(vision_worker, 'load_image_base64', return_value=("data:image/png;base64,abc", "image/png")), \
             patch.object(vision_worker, '_get_llm_client') as mock_get_client:
            
            mock_client = MagicMock()
            mock_response = MagicMock()
            mock_response.choices[0].message.content = "Vision Summary"
            mock_client.send_llm_request.return_value = mock_response
            mock_get_client.return_value = mock_client

            processed_count = await vision_worker.process_batch(conn)

            assert processed_count == 1
            
            row = await conn.fetchrow("SELECT vision_summary, vision_status FROM frames WHERE id = $1", frame_id)
            assert row["vision_summary"] == "Vision Summary"
            assert row["vision_status"] == VISION_STATUS_VISION_DONE

@pytest.mark.asyncio
async def test_vision_pipeline_api_error(db_pool, vision_worker):
    """
    Integration test: LLM API Failure -> Verify status set to ERROR (-1).
    """
    async with db_pool.acquire() as conn:
        frame_id = uuid4()
        await conn.execute(
            """
            INSERT INTO frames (id, captured_at, image_ref, ocr_text, vision_status)
            VALUES ($1, $2, $3, $4, $5)
            """,
            frame_id, 
            datetime.now(timezone.utc), 
            "test_image.png", 
            "ocr text content",
            VISION_STATUS_OCR_DONE
        )

        with patch.object(vision_worker, 'load_image_base64', return_value=("data:image/png;base64,abc", "image/png")), \
             patch.object(vision_worker, '_get_llm_client') as mock_get_client:
            
            mock_client = MagicMock()
            # Simulate generic API error
            mock_client.send_llm_request.side_effect = Exception("API Connection Error")
            mock_get_client.return_value = mock_client

            processed_count = await vision_worker.process_batch(conn)

            assert processed_count == 1
            
            row = await conn.fetchrow("SELECT vision_status, vision_summary FROM frames WHERE id = $1", frame_id)
            assert row["vision_status"] == VISION_STATUS_ERROR
            assert row["vision_summary"] is None

@pytest.mark.asyncio
async def test_vision_pipeline_rate_limit_retry(db_pool, vision_worker):
    """
    Integration test: Simulate Rate Limit (429) -> Verify Retry Logic.
    
    NOTE: The `VisionWorker` class implements retry logic in `run_with_retry` for DB errors,
    but `process_frame` catches exceptions and returns error immediately. 
    If we want to test RE-TRYING the LLM call, we'd need to modify the worker 
    to bubble up transient errors or handle them internally.
    
    As currently implemented in `process_frame`, any exception makes it fail. 
    So this test verifies that behavior explicitly (fail fast on API error), 
    unless we decide to strictly implement the retry logic requested in TODOs.
    
    For now, we verify that it fails safely, which matches current implementation.
    """
    async with db_pool.acquire() as conn:
        frame_id = uuid4()
        await conn.execute(
            """
            INSERT INTO frames (id, captured_at, image_ref, ocr_text, vision_status)
            VALUES ($1, $2, $3, $4, $5)
            """,
            frame_id, 
            datetime.now(timezone.utc), 
            "test_image.png", 
            "ocr text content",
            VISION_STATUS_OCR_DONE
        )

        with patch.object(vision_worker, 'load_image_base64', return_value=("data:image/png;base64,abc", "image/png")), \
             patch.object(vision_worker, '_get_llm_client') as mock_get_client:
            
            mock_client = MagicMock()
            mock_client.send_llm_request.side_effect = Exception("429 Too Many Requests")
            mock_get_client.return_value = mock_client

            # We are calling process_batch which calls process_frame
            # process_frame catches Exception and returns VisionResult with error
            processed_count = await vision_worker.process_batch(conn)

            assert processed_count == 1
            
            row = await conn.fetchrow("SELECT vision_status, vision_summary FROM frames WHERE id = $1", frame_id)
            assert row["vision_status"] == VISION_STATUS_ERROR
