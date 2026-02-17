import pytest
import os
from uuid import uuid4
from datetime import datetime, timezone

@pytest.fixture
def db_url():
    """
    Returns the DATABASE_URL environment variable.
    Skips the test if the variable is not set.
    """
    url = os.getenv("DATABASE_URL")
    if not url:
        pytest.skip("DATABASE_URL not set")
    return url

@pytest.fixture
def sample_frame_data():
    """
    Returns a dictionary with valid Frame field values.
    """
    return {
        "id": uuid4(),
        "captured_at": datetime.now(timezone.utc),
        "window_title": "Test Window",
        "app_name": "Test App",
        "image_ref": "2025-01-01/test.jpg",
        "phash64": 1234567890,
        "ocr_text": "Sample text",
        "vision_summary": "A test window",
        "ocr_status": 0,
        "vision_status": 0,
        "embedding_status": 0
    }
