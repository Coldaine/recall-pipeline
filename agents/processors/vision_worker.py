"""
LLM Vision Summarization Worker for Recall Pipeline.

Polls for OCR-processed frames and generates concise summaries using LLM Vision APIs.
Updates the database with vision summaries and status.

Vision status values:
    0 = pending (unprocessed)
    1 = processing (OCR in progress)
    2 = OCR done (ready for vision)
    3 = vision processing
    4 = vision done
    -1 = error (failed processing)
"""

import argparse
import asyncio
import base64
import logging
import mimetypes
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from uuid import UUID

import asyncpg

from agents.database.connection import get_db_connection
from agents.llm_api.llm_client import LLMClient
from agents.schemas.agents_message_content import ImageContent, TextContent
from agents.schemas.enums import MessageRole
from agents.schemas.llm_config import LLMConfig
from agents.schemas.message import Message

logger = logging.getLogger(__name__)

# Vision status constants
VISION_STATUS_PENDING = 0
VISION_STATUS_PROCESSING = 1
VISION_STATUS_OCR_DONE = 2
VISION_STATUS_VISION_PROCESSING = 3
VISION_STATUS_VISION_DONE = 4
VISION_STATUS_ERROR = -1

# Default vision prompt
DEFAULT_VISION_PROMPT = """You are analyzing a screenshot from a user's computer.
The OCR extracted text is: {ocr_text}

Describe concisely (1-2 sentences) what application/window is visible and what the user is likely doing. Focus on the activity, not UI elements."""


@dataclass
class VisionResult:
    """Result of vision processing for a single frame."""

    frame_id: UUID
    summary: str | None = None
    error: str | None = None


@dataclass
class VisionFrameRecord:
    """Minimal frame record for vision processing."""

    id: UUID
    captured_at: Any  # datetime
    image_ref: str
    ocr_text: str | None = None
    vision_status: int = 0


class VisionWorker:
    """
    Long-running worker that polls for OCR-processed frames and generates summaries.

    Uses LLM Vision APIs (OpenAI GPT-4V, Anthropic Claude Vision) to analyze
    screenshots and generate concise summaries of what's on screen.

    Example:
        worker = VisionWorker(batch_size=10, poll_interval=5.0)
        await worker.run()
    """

    def __init__(
        self,
        batch_size: int = 10,
        poll_interval: float = 5.0,
        max_retries: int = 3,
        retry_delay: float = 1.0,
        model: str = "gpt-4o",
        model_endpoint: str | None = None,
        max_tokens: int = 150,
        vision_prompt: str | None = None,
        rate_limit_delay: float = 0.5,
    ):
        """
        Initialize the Vision worker.

        Args:
            batch_size: Number of frames to process per batch.
            poll_interval: Seconds to wait between polling cycles.
            max_retries: Maximum retry attempts for database errors.
            retry_delay: Base delay between retries (exponential backoff).
            model: LLM model to use for vision (e.g., 'gpt-4o', 'claude-3-5-sonnet-latest').
            model_endpoint: Optional custom model endpoint URL.
            max_tokens: Maximum tokens for LLM response.
            vision_prompt: Custom prompt template for vision analysis.
            rate_limit_delay: Delay between API calls to avoid rate limits.
        """
        self.batch_size = batch_size
        self.poll_interval = poll_interval
        self.max_retries = max_retries
        self.retry_delay = retry_delay
        self.model = model
        self.model_endpoint = model_endpoint
        self.max_tokens = max_tokens
        self.vision_prompt = vision_prompt or DEFAULT_VISION_PROMPT
        self.rate_limit_delay = rate_limit_delay
        self.running = False
        self._llm_client: LLMClient | None = None

    def _get_llm_client(self) -> LLMClient:
        """Get or create the LLM client."""
        if self._llm_client is None:
            config = LLMConfig.default_config(self.model)
            if self.model_endpoint:
                config.model_endpoint = self.model_endpoint
            config.max_tokens = self.max_tokens
            self._llm_client = LLMClient.create(config)
            if not self._llm_client:
                raise ValueError(f"Failed to create LLM client for model {self.model}")
        return self._llm_client

    async def fetch_ocr_done_frames(
        self, conn: asyncpg.Connection
    ) -> list[VisionFrameRecord]:
        """
        Fetch frames that have completed OCR and need vision processing.

        Uses FOR UPDATE SKIP LOCKED to allow multiple workers safely.

        Args:
            conn: asyncpg database connection.

        Returns:
            List of VisionFrameRecord objects for processing.
        """
        rows = await conn.fetch(
            """
            SELECT id, captured_at, image_ref, ocr_text, vision_status
            FROM frames
            WHERE vision_status = $1
            ORDER BY captured_at ASC
            LIMIT $2
            FOR UPDATE SKIP LOCKED
            """,
            VISION_STATUS_OCR_DONE,
            self.batch_size,
        )
        return [VisionFrameRecord(**dict(row)) for row in rows]

    async def mark_frames_processing(
        self, conn: asyncpg.Connection, frame_ids: list[UUID]
    ) -> None:
        """Mark frames as being processed to prevent other workers from picking them up."""
        if not frame_ids:
            return
        await conn.execute(
            """
            UPDATE frames
            SET vision_status = $1
            WHERE id = ANY($2::uuid[])
            """,
            VISION_STATUS_VISION_PROCESSING,
            frame_ids,
        )

    def load_image_base64(self, image_ref: str) -> tuple[str, str] | None:
        """
        Load an image and encode it as base64.

        Args:
            image_ref: Path or URI to the image file.

        Returns:
            Tuple of (base64_data_uri, mime_type) or None if loading fails.
        """
        try:
            # Handle file:// URIs
            if image_ref.startswith("file://"):
                image_ref = image_ref[7:]

            path = Path(image_ref)

            # Check if path exists
            if not path.exists():
                logger.warning(f"Image file not found: {path}")
                return None

            # Determine MIME type
            mime_type, _ = mimetypes.guess_type(str(path))
            if mime_type is None or not mime_type.startswith("image/"):
                mime_type = "image/jpeg"

            # Read and encode
            with open(path, "rb") as img_file:
                base64_string = base64.b64encode(img_file.read()).decode("utf-8")
                data_uri = f"data:{mime_type};base64,{base64_string}"
                return data_uri, mime_type

        except Exception as e:
            logger.error(f"Failed to load image {image_ref}: {e}")
            return None

    def generate_summary(
        self, image_data_uri: str, ocr_text: str | None
    ) -> str | None:
        """
        Generate a summary for the frame using the LLM Vision API.

        Args:
            image_data_uri: Base64-encoded image data URI.
            ocr_text: OCR text extracted from the image (may be None).

        Returns:
            Generated summary string or None on failure.
        """
        try:
            client = self._get_llm_client()

            # Format the prompt with OCR text
            ocr_context = ocr_text[:1000] if ocr_text else "(no text detected)"
            prompt_text = self.vision_prompt.format(ocr_text=ocr_context)

            # Construct the message with image
            message = Message(
                role=MessageRole.user,
                content=[
                    TextContent(text=prompt_text),
                    ImageContent(image_id=image_data_uri),
                ],
            )

            # Call the LLM
            response = client.send_llm_request(messages=[message])

            # Extract the summary from response
            if response and response.choices:
                return response.choices[0].message.content

            return None

        except Exception as e:
            logger.error(f"Failed to generate vision summary: {e}")
            return None

    async def update_frame_result(
        self,
        conn: asyncpg.Connection,
        result: VisionResult,
    ) -> None:
        """
        Update the database with vision processing results.

        Args:
            conn: asyncpg database connection.
            result: Vision processing result.
        """
        if result.error:
            await conn.execute(
                """
                UPDATE frames
                SET vision_status = $1
                WHERE id = $2
                """,
                VISION_STATUS_ERROR,
                result.frame_id,
            )
            logger.error(f"Frame {result.frame_id} marked as error: {result.error}")
            return

        await conn.execute(
            """
            UPDATE frames
            SET vision_summary = $1, vision_status = $2
            WHERE id = $3
            """,
            result.summary,
            VISION_STATUS_VISION_DONE,
            result.frame_id,
        )

        logger.info(
            f"Frame {result.frame_id} processed: "
            f"summary_len={len(result.summary) if result.summary else 0}"
        )

    async def process_frame(self, frame: VisionFrameRecord) -> VisionResult:
        """
        Process a single frame with LLM Vision.

        Args:
            frame: Frame record to process.

        Returns:
            VisionResult with generated summary or error.
        """
        try:
            # Load the image
            image_result = self.load_image_base64(frame.image_ref)
            if image_result is None:
                return VisionResult(
                    frame_id=frame.id,
                    error=f"Could not load image: {frame.image_ref}",
                )

            image_data_uri, _ = image_result

            # Generate summary
            summary = self.generate_summary(image_data_uri, frame.ocr_text)

            if summary is None:
                return VisionResult(
                    frame_id=frame.id,
                    error="LLM Vision API returned no summary",
                )

            return VisionResult(
                frame_id=frame.id,
                summary=summary,
            )

        except Exception as e:
            logger.exception(f"Error processing frame {frame.id}")
            return VisionResult(
                frame_id=frame.id,
                error=str(e),
            )

    async def process_batch(self, conn: asyncpg.Connection) -> int:
        """
        Process a batch of frames.

        Args:
            conn: asyncpg database connection.

        Returns:
            Number of frames processed.
        """
        # Fetch OCR-done frames
        frames = await self.fetch_ocr_done_frames(conn)
        if not frames:
            return 0

        logger.info(f"Processing {len(frames)} frames with LLM Vision")

        # Mark as vision processing
        frame_ids = [f.id for f in frames]
        await self.mark_frames_processing(conn, frame_ids)

        # Process each frame with rate limiting
        results = []
        for i, frame in enumerate(frames):
            result = await self.process_frame(frame)
            results.append(result)

            # Add delay between API calls to avoid rate limits
            if i < len(frames) - 1:
                await asyncio.sleep(self.rate_limit_delay)

        # Update results in a transaction
        async with conn.transaction():
            for result in results:
                await self.update_frame_result(conn, result)

        return len(frames)

    async def run_with_retry(self, conn: asyncpg.Connection) -> int:
        """
        Run a processing cycle with retry logic for transient errors.

        Args:
            conn: asyncpg database connection.

        Returns:
            Number of frames processed.
        """
        last_error = None
        for attempt in range(self.max_retries):
            try:
                return await self.process_batch(conn)
            except asyncpg.PostgresError as e:
                last_error = e
                delay = self.retry_delay * (2**attempt)
                logger.warning(
                    f"Database error (attempt {attempt + 1}/{self.max_retries}): {e}. "
                    f"Retrying in {delay}s"
                )
                await asyncio.sleep(delay)
            except Exception as e:
                logger.error(f"Unexpected error in processing cycle: {e}")
                raise

        logger.error(f"Max retries exceeded. Last error: {last_error}")
        return 0

    async def run(self) -> None:
        """
        Main worker loop.

        Continuously polls for OCR-processed frames and generates vision summaries.
        Runs until stopped via the running flag or KeyboardInterrupt.
        """
        self.running = True
        logger.info(
            f"Vision Worker started (model={self.model}, batch_size={self.batch_size}, "
            f"poll_interval={self.poll_interval}s)"
        )

        # Initialize LLM client at startup
        try:
            self._get_llm_client()
            logger.info(f"LLM client initialized for model: {self.model}")
        except Exception as e:
            logger.error(f"Failed to initialize LLM client: {e}")
            self.running = False
            return

        while self.running:
            try:
                async with get_db_connection() as conn:
                    processed = await self.run_with_retry(conn)

                    if processed == 0:
                        # No frames to process, wait before next poll
                        await asyncio.sleep(self.poll_interval)
                    else:
                        # Processed frames, immediately check for more
                        logger.info(f"Processed {processed} frames with vision")

            except asyncio.CancelledError:
                logger.info("Worker cancelled")
                self.running = False
                break
            except Exception as e:
                logger.exception(f"Error in worker loop: {e}")
                await asyncio.sleep(self.poll_interval)

        logger.info("Vision Worker stopped")

    def stop(self) -> None:
        """Signal the worker to stop gracefully."""
        logger.info("Stopping Vision Worker...")
        self.running = False


async def main() -> None:
    """Entry point for running the Vision worker as a standalone process."""
    parser = argparse.ArgumentParser(
        description="LLM Vision Summarization Worker for Recall Pipeline"
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=10,
        help="Number of frames to process per batch (default: 10)",
    )
    parser.add_argument(
        "--poll-interval",
        type=float,
        default=5.0,
        help="Seconds to wait between polling cycles (default: 5.0)",
    )
    parser.add_argument(
        "--model",
        default="gpt-4o",
        help="LLM model for vision (default: gpt-4o)",
    )
    parser.add_argument(
        "--model-endpoint",
        default=None,
        help="Custom model endpoint URL (optional)",
    )
    parser.add_argument(
        "--max-tokens",
        type=int,
        default=150,
        help="Maximum tokens for LLM response (default: 150)",
    )
    parser.add_argument(
        "--rate-limit-delay",
        type=float,
        default=0.5,
        help="Delay between API calls in seconds (default: 0.5)",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Enable verbose logging",
    )

    args = parser.parse_args()

    # Configure logging
    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )

    worker = VisionWorker(
        batch_size=args.batch_size,
        poll_interval=args.poll_interval,
        model=args.model,
        model_endpoint=args.model_endpoint,
        max_tokens=args.max_tokens,
        rate_limit_delay=args.rate_limit_delay,
    )

    try:
        await worker.run()
    except KeyboardInterrupt:
        worker.stop()


if __name__ == "__main__":
    asyncio.run(main())
