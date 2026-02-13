from datetime import datetime
from typing import Optional, List
from pydantic import BaseModel, Field, ConfigDict
from uuid import UUID

class FrameBase(BaseModel):
    captured_at: datetime
    window_title: Optional[str] = None
    app_name: Optional[str] = None
    image_ref: Optional[str] = None
    phash64: Optional[int] = None
    ocr_text: Optional[str] = None
    vision_summary: Optional[str] = None
    
    model_config = ConfigDict(from_attributes=True)

class Frame(FrameBase):
    id: UUID
    ocr_status: int = 0  # 0=pending, 1=processed, 2=failed
    vision_status: int = 0
    embedding_status: int = 0
    created_at: datetime

class VisionSummaryRequest(BaseModel):
    frame_id: UUID
    image_path: str

class OCRResult(BaseModel):
    text: str
    confidence: Optional[float] = None
