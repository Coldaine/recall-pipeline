import pytest
from uuid import uuid4
from datetime import datetime, timezone
from agents.processors.ocr_worker import OCRResult, FrameRecord, OCRWorker

def test_ocr_result_creation():
    """Test OCRResult dataclass."""
    frame_id = uuid4()
    res = OCRResult(frame_id=frame_id, text="sample")
    assert res.text == "sample"
    assert res.confidence is None

def test_frame_record_creation():
    """Test FrameRecord dataclass."""
    frame_id = uuid4()
    now = datetime.now(timezone.utc)
    rec = FrameRecord(id=frame_id, captured_at=now, image_ref="img.jpg")
    assert rec.image_ref == "img.jpg"
    assert rec.vision_status == 0

def test_worker_init_defaults(db_url):
    """Test OCRWorker initialization with defaults."""
    # We need a DB URL even for init because it might set up connection pools
    # But usually init is lazy. Let's see.
    # The worker init connects to DB in `run`, but `__init__` might be safe.
    # Looking at code: __init__ just stores args.
    worker = OCRWorker(db_url)
    assert worker.batch_size == 10
    assert worker.interval == 1.0

def test_worker_init_custom(db_url):
    """Test OCRWorker initialization with custom values."""
    worker = OCRWorker(db_url, batch_size=50, interval=5.0)
    assert worker.batch_size == 50
    assert worker.interval == 5.0
