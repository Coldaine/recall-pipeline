from agents.schemas.frame import Frame, FrameBase
from uuid import uuid4
from datetime import datetime, timezone

def test_frame_base_creation():
    """Test that FrameBase can be created with minimal fields."""
    now = datetime.now(timezone.utc)
    fb = FrameBase(captured_at=now)
    assert fb.captured_at == now
    assert fb.window_title is None

def test_frame_creation(sample_frame_data):
    """Test that Frame can be created with all fields."""
    frame = Frame(created_at=datetime.now(timezone.utc), **sample_frame_data)
    
    assert frame.id == sample_frame_data["id"]
    assert frame.app_name == "Test App"
    assert frame.ocr_status == 0

def test_frame_defaults():
    """Test default values for Frame status fields."""
    now = datetime.now(timezone.utc)
    frame = Frame(
        id=uuid4(),
        captured_at=now,
        created_at=now
    )
    assert frame.ocr_status == 0
    assert frame.vision_status == 0
    assert frame.embedding_status == 0
