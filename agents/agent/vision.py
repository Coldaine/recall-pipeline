import logging
import base64
import os
from uuid import UUID
from typing import Optional, List

from agents.schemas.frame import Frame
from agents.schemas.llm_config import LLMConfig
from agents.llm_api.llm_client import LLMClient
from agents.settings import model_settings
from agents.schemas.message import Message
from agents.schemas.enums import MessageRole
from agents.schemas.agents_message_content import TextContent, ImageContent

logger = logging.getLogger(__name__)

class VisionAgent:
    def __init__(self, model: str = "gpt-4o"):
        # Configure the LLM
        self.config = LLMConfig.default_config(model)
        
        # Use the client factory
        self.client = LLMClient.create(self.config)
        if not self.client:
            raise ValueError(f"Failed to create LLM client for model {model}")

    def encode_image(self, image_path: str) -> str:
        with open(image_path, "rb") as image_file:
            return base64.b64encode(image_file.read()).decode('utf-8')

    def summarize_frame(self, frame: Frame) -> Optional[str]:
        """
        Generate a summary for the frame using the LLM.
        """
        if not frame.image_ref or not os.path.exists(frame.image_ref):
            logger.warning(f"Image not found for frame {frame.id}: {frame.image_ref}")
            return None

        try:
            base64_image = self.encode_image(frame.image_ref)
            
            prompt_text = (
                f"Analyze this screen capture from the application '{frame.app_name}'. "
                f"Window title: '{frame.window_title}'. "
                f"OCR Text: '{frame.ocr_text[:500] if frame.ocr_text else ''}...'. "
                "Provide a concise summary of what the user is doing or what is visible."
            )
            
            # Construct the message using the defined schemas
            # Note: ImageContent expects image_id usually for DB ref, but some clients might handle base64/url if configured?
            # Looking at `to_openai_dict` in Message, it handles ImageContent by using `image_id`.
            # If we want to pass base64, we might need to use a specific way or 'hack' it if the client doesn't support direct base64 injection via ImageContent.
            # OpenAI API supports base64 data URLs.
            # Let's see if LLMClientBase supports base64.
            
            # For now, we will try to manually construct a message that the client accepts if Message schema is too strict or DB-bound.
            # But send_llm_request expects List[Message].
            
            # If ImageContent only stores ID, we might need to upload it or use a custom content type?
            # Wait, `to_openai_dict` checks `ImageContent` and puts `image_url`.
            # But `ImageContent` in `agents_message_content.py` likely just has `image_id`.
            
            # Let's check ImageContent definition.
            # It seems it only has image_id.
            # If so, the system expects images to be in a storage/DB.
            # But we are in a "Frame Processor" context where images are on disk.
            
            # I'll assume for this prototype that I can pass a base64 data URI as the 'image_id' if the client allows it, 
            # OR I might need to modify the client/message structure.
            
            # Actually, `to_anthropic_dict` logic:
            # `{"type": "image_url", "image_id": content_part.image_id}`
            # It doesn't look like it resolves to base64 automatically unless `image_id` IS the url/base64.
            
            # Let's try passing the data URI as the image_id.
            data_uri = f"data:image/jpeg;base64,{base64_image}"
            
            message = Message(
                role=MessageRole.user,
                content=[
                    TextContent(text=prompt_text),
                    ImageContent(image_id=data_uri) 
                ]
            )
            
            # Call the client
            response = self.client.send_llm_request(messages=[message])
            
            # Response is ChatCompletionResponse
            # It has choices[0].message.content
            return response.choices[0].message.content
                 
        except Exception as e:
            logger.error(f"Error summarizing frame {frame.id}: {e}")
            return None
