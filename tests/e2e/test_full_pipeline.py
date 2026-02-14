import pytest
import asyncpg
from uuid import uuid4
from datetime import datetime, timezone
from unittest.mock import MagicMock, patch
from agents.processors.ocr_worker import OCRWorker, VISION_STATUS_PENDING, VISION_STATUS_DONE, VISION_STATUS_ERROR
from agents.processors.vision_worker import VisionWorker, VISION_STATUS_OCR_DONE, VISION_STATUS_VISION_DONE

@pytest.mark.asyncio
async def test_full_chain_pipeline(db_pool):
    """
    E2E Test: Simulate "Capture" -> Run OCR -> Run Vision -> Verify Final State.
    """
    # 1. Initialize Workers
    ocr_worker = OCRWorker(batch_size=1, poll_interval=0.1)
    vision_worker = VisionWorker(batch_size=1, poll_interval=0.1, model="test-model")

    async with db_pool.acquire() as conn:
        # 2. Simulate Capture (Insert Frame)
        frame_id = uuid4()
        await conn.execute(
            """
            INSERT INTO frames (id, captured_at, image_ref, vision_status)
            VALUES ($1, $2, $3, $4)
            """,
            frame_id, 
            datetime.now(timezone.utc), 
            "e2e_test_image.png", 
            VISION_STATUS_PENDING
        )

        # 3. Operations - Step 1: OCR
        # Mock OCR Internals
        with patch.object(ocr_worker, 'load_image', return_value="ImageObj"), \
             patch.object(ocr_worker, 'run_ocr', return_value=("E2E OCR Text", 0.98)):
            
            ocr_processed = await ocr_worker.process_batch(conn)
            assert ocr_processed == 1

        # Verify Middle State
        row = await conn.fetchrow("SELECT vision_status, ocr_text FROM frames WHERE id = $1", frame_id)
        assert row["vision_status"] == VISION_STATUS_DONE # This is 2, which matches VISION_STATUS_OCR_DONE
        assert row["ocr_text"] == "E2E OCR Text"

        # 4. Operations - Step 2: Vision
        # Mock Vision Internals
        with patch.object(vision_worker, 'load_image_base64', return_value=("data:image/png;base64,yyy", "image/png")), \
             patch.object(vision_worker, '_get_llm_client') as mock_get_client:
            
            mock_client = MagicMock()
            mock_response = MagicMock()
            mock_response.choices[0].message.content = "E2E Vision Summary"
            mock_client.send_llm_request.return_value = mock_response
            mock_get_client.return_value = mock_client

            vision_processed = await vision_worker.process_batch(conn)
            assert vision_processed == 1

        # 5. Verify Final State
        row = await conn.fetchrow("SELECT vision_status, vision_summary FROM frames WHERE id = $1", frame_id)
        assert row["vision_status"] == VISION_STATUS_VISION_DONE
        assert row["vision_summary"] == "E2E Vision Summary"

