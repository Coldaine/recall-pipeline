import pytest
import asyncpg
from uuid import uuid4
from datetime import datetime, timezone
from agents.processors.ocr_worker import OCRWorker, FrameRecord, VISION_STATUS_PENDING, VISION_STATUS_DONE, VISION_STATUS_ERROR

@pytest.fixture
def ocr_worker():
    # Initialize worker with test settings
    return OCRWorker(
        batch_size=1,
        poll_interval=0.1,
        max_retries=1
    )

@pytest.mark.asyncio
async def test_ocr_pipeline_happy_path(db_pool, ocr_worker):
    """
    Integration test: Insert a valid frame -> Run OCR -> Verify DB update.
    
    NOTE: This test currently assumes we can mock the actual image loading and OCR 
    part within the worker while testing the DB interaction. 
    Ideally, we'd use a real image, but to keep this test portable without 
    Tesseract installed on the runner, we mock the `run_ocr` and `load_image` methods.
    """
    async with db_pool.acquire() as conn:
        # 1. Setup Data
        frame_id = uuid4()
        await conn.execute(
            """
            INSERT INTO frames (id, captured_at, image_ref, vision_status)
            VALUES ($1, $2, $3, $4)
            """,
            frame_id, 
            datetime.now(timezone.utc), 
            "test_image.png", 
            VISION_STATUS_PENDING
        )

        # 2. Execute Worker Logic (Partial Mocking)
        # We want to test the `process_batch` DB logic, but mock the heavy lifting
        from unittest.mock import MagicMock
        
        # Mock load_image to return a dummy object
        ocr_worker.load_image = MagicMock(return_value="Valid Image Object")
        # Mock run_ocr to return valid text
        ocr_worker.run_ocr = MagicMock(return_value=("Integration Test Text", 0.99))

        processed_count = await ocr_worker.process_batch(conn)

        # 3. Verify
        assert processed_count == 1
        
        row = await conn.fetchrow("SELECT ocr_text, vision_status, has_text FROM frames WHERE id = $1", frame_id)
        assert row["ocr_text"] == "Integration Test Text"
        assert row["vision_status"] == VISION_STATUS_DONE
        assert row["has_text"] is True

@pytest.mark.asyncio
async def test_ocr_pipeline_file_not_found(db_pool, ocr_worker):
    """
    Integration test: Frame with invalid path -> Verify status set to ERROR (-1).
    """
    async with db_pool.acquire() as conn:
        frame_id = uuid4()
        await conn.execute(
            """
            INSERT INTO frames (id, captured_at, image_ref, vision_status)
            VALUES ($1, $2, $3, $4)
            """,
            frame_id, 
            datetime.now(timezone.utc), 
            "non_existent_file.png", 
            VISION_STATUS_PENDING
        )

        # Ensure load_image returns None (simulation of failure)
        ocr_worker.load_image = MagicMock(return_value=None)

        processed_count = await ocr_worker.process_batch(conn)

        assert processed_count == 1
        
        row = await conn.fetchrow("SELECT vision_status FROM frames WHERE id = $1", frame_id)
        assert row["vision_status"] == VISION_STATUS_ERROR

@pytest.mark.asyncio
async def test_ocr_pipeline_corrupt_image(db_pool, ocr_worker):
    """
    Integration test: OCR failure (e.g., corrupt image) -> Verify status set to ERROR (-1).
    """
    async with db_pool.acquire() as conn:
        frame_id = uuid4()
        await conn.execute(
            """
            INSERT INTO frames (id, captured_at, image_ref, vision_status)
            VALUES ($1, $2, $3, $4)
            """,
            frame_id, 
            datetime.now(timezone.utc), 
            "corrupt_image.png", 
            VISION_STATUS_PENDING
        )

        # Mock load_image to succeed
        ocr_worker.load_image = MagicMock(return_value="Corrupt Image Object")
        # Mock run_ocr to RAISE an exception
        ocr_worker.run_ocr = MagicMock(side_effect=Exception("Corrupt image file"))

        processed_count = await ocr_worker.process_batch(conn)

        assert processed_count == 1
        
        row = await conn.fetchrow("SELECT vision_status FROM frames WHERE id = $1", frame_id)
        assert row["vision_status"] == VISION_STATUS_ERROR
