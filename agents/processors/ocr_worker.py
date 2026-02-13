"""
OCR Worker for Recall Pipeline.

Polls for unprocessed frames and runs Tesseract OCR to extract text.
Updates the database with OCR results and status.

Vision status values:
    0 = pending (unprocessed)
    1 = processing (currently being worked on)
    2 = done (successfully processed)
    -1 = error (failed processing)
"""

import asyncio
import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from uuid import UUID

import asyncpg
from PIL import Image

from agents.database.connection import get_db_connection

logger = logging.getLogger(__name__)

# Vision status constants
VISION_STATUS_PENDING = 0
VISION_STATUS_PROCESSING = 1
VISION_STATUS_DONE = 2
VISION_STATUS_ERROR = -1


@dataclass
class OCRResult:
    """Result of OCR processing for a single frame."""

    frame_id: UUID
    text: str
    confidence: float | None = None
    language: str | None = None
    error: str | None = None


@dataclass
class FrameRecord:
    """Minimal frame record for OCR processing."""

    id: UUID
    captured_at: Any  # datetime
    image_ref: str
    vision_status: int = 0


class OCRWorker:
    """
    Long-running worker that polls for unprocessed frames and runs OCR.

    Uses Tesseract via pytesseract for text extraction. Designed to work
    with the recall-pipeline frames table in PostgreSQL.

    Example:
        worker = OCRWorker(batch_size=10, poll_interval=5.0)
        await worker.run()
    """

    def __init__(
        self,
        batch_size: int = 10,
        poll_interval: float = 5.0,
        max_retries: int = 3,
        retry_delay: float = 1.0,
        tesseract_lang: str = "eng",
        tesseract_config: str = "",
        min_text_length: int = 1,
    ):
        """
        Initialize the OCR worker.

        Args:
            batch_size: Number of frames to process per batch.
            poll_interval: Seconds to wait between polling cycles.
            max_retries: Maximum retry attempts for database errors.
            retry_delay: Base delay between retries (exponential backoff).
            tesseract_lang: Language for Tesseract OCR (e.g., 'eng', 'eng+spa').
            tesseract_config: Additional Tesseract configuration string.
            min_text_length: Minimum text length to consider as having text.
        """
        self.batch_size = batch_size
        self.poll_interval = poll_interval
        self.max_retries = max_retries
        self.retry_delay = retry_delay
        self.tesseract_lang = tesseract_lang
        self.tesseract_config = tesseract_config
        self.min_text_length = min_text_length
        self.running = False
        self._tesseract_available: bool | None = None

    def _check_tesseract(self) -> bool:
        """Check if Tesseract is available and cache the result."""
        if self._tesseract_available is None:
            try:
                import pytesseract

                # Try to get Tesseract version to verify it's installed
                pytesseract.get_tesseract_version()
                self._tesseract_available = True
                logger.info("Tesseract OCR is available")
            except Exception as e:
                self._tesseract_available = False
                logger.warning(f"Tesseract OCR not available: {e}")
        return self._tesseract_available

    async def fetch_pending_frames(self, conn: asyncpg.Connection) -> list[FrameRecord]:
        """
        Fetch frames that need OCR processing.

        Uses FOR UPDATE SKIP LOCKED to allow multiple workers safely.

        Args:
            conn: asyncpg database connection.

        Returns:
            List of FrameRecord objects for processing.
        """
        rows = await conn.fetch(
            """
            SELECT id, captured_at, image_ref, vision_status
            FROM frames
            WHERE vision_status = $1
            ORDER BY captured_at ASC
            LIMIT $2
            FOR UPDATE SKIP LOCKED
            """,
            VISION_STATUS_PENDING,
            self.batch_size,
        )
        return [FrameRecord(**dict(row)) for row in rows]

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
            VISION_STATUS_PROCESSING,
            frame_ids,
        )

    def load_image(self, image_ref: str) -> Image.Image | None:
        """
        Load an image from the given reference.

        Supports:
        - Local file paths (absolute or relative)
        - file:// URIs

        Args:
            image_ref: Path or URI to the image file.

        Returns:
            PIL Image object or None if loading fails.
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

            # Load and return the image
            return Image.open(path)

        except Exception as e:
            logger.error(f"Failed to load image {image_ref}: {e}")
            return None

    def run_ocr(self, image: Image.Image) -> tuple[str, float | None]:
        """
        Run Tesseract OCR on an image.

        Args:
            image: PIL Image to process.

        Returns:
            Tuple of (extracted_text, confidence) or ("", None) on failure.
        """
        if not self._check_tesseract():
            raise RuntimeError("Tesseract OCR is not available")

        import pytesseract

        try:
            # Get OCR data with confidence
            data = pytesseract.image_to_data(
                image,
                lang=self.tesseract_lang,
                config=self.tesseract_config,
                output_type=pytesseract.Output.DICT,
            )

            # Extract text and calculate average confidence
            texts = []
            confidences = []

            for i, text in enumerate(data.get("text", [])):
                if text.strip():
                    texts.append(text)
                    conf = data.get("conf", [])[i]
                    if conf > 0:  # Valid confidence value
                        confidences.append(conf)

            full_text = " ".join(texts)
            avg_confidence = sum(confidences) / len(confidences) if confidences else None

            return full_text, avg_confidence

        except Exception as e:
            logger.error(f"OCR failed: {e}")
            return "", None

    async def update_frame_result(
        self,
        conn: asyncpg.Connection,
        result: OCRResult,
    ) -> None:
        """
        Update the database with OCR results.

        Updates:
        - ocr_text column with extracted text
        - has_text flag based on text content
        - vision_status to done or error

        Args:
            conn: asyncpg database connection.
            result: OCR processing result.
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

        has_text = len(result.text.strip()) >= self.min_text_length

        await conn.execute(
            """
            UPDATE frames
            SET ocr_text = $1, has_text = $2, vision_status = $3
            WHERE id = $4
            """,
            result.text if has_text else None,
            has_text,
            VISION_STATUS_DONE,
            result.frame_id,
        )

        # Also insert into ocr_text table for detailed records
        if has_text:
            await conn.execute(
                """
                INSERT INTO ocr_text (frame_id, text, confidence, language)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT DO NOTHING
                """,
                result.frame_id,
                result.text,
                result.confidence,
                result.language,
            )

        logger.info(
            f"Frame {result.frame_id} processed: "
            f"has_text={has_text}, text_len={len(result.text)}"
        )

    async def process_frame(self, frame: FrameRecord) -> OCRResult:
        """
        Process a single frame with OCR.

        Args:
            frame: Frame record to process.

        Returns:
            OCRResult with extracted text or error.
        """
        try:
            # Load the image
            image = self.load_image(frame.image_ref)
            if image is None:
                return OCRResult(
                    frame_id=frame.id,
                    text="",
                    error=f"Could not load image: {frame.image_ref}",
                )

            # Run OCR
            text, confidence = self.run_ocr(image)

            return OCRResult(
                frame_id=frame.id,
                text=text,
                confidence=confidence,
                language=self.tesseract_lang,
            )

        except Exception as e:
            logger.exception(f"Error processing frame {frame.id}")
            return OCRResult(
                frame_id=frame.id,
                text="",
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
        # Fetch pending frames
        frames = await self.fetch_pending_frames(conn)
        if not frames:
            return 0

        logger.info(f"Processing {len(frames)} frames")

        # Mark as processing
        frame_ids = [f.id for f in frames]
        await self.mark_frames_processing(conn, frame_ids)

        # Process each frame
        results = []
        for frame in frames:
            result = await self.process_frame(frame)
            results.append(result)

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

        Continuously polls for unprocessed frames and runs OCR.
        Runs until stopped via the running flag or KeyboardInterrupt.
        """
        self.running = True
        logger.info(
            f"OCR Worker started (batch_size={self.batch_size}, "
            f"poll_interval={self.poll_interval}s)"
        )

        # Check Tesseract availability at startup
        if not self._check_tesseract():
            logger.error(
                "Tesseract OCR is not available. "
                "Install with: apt install tesseract-ocr libtesseract-dev"
            )
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
                        logger.info(f"Processed {processed} frames")

            except asyncio.CancelledError:
                logger.info("Worker cancelled")
                self.running = False
                break
            except Exception as e:
                logger.exception(f"Error in worker loop: {e}")
                await asyncio.sleep(self.poll_interval)

        logger.info("OCR Worker stopped")

    def stop(self) -> None:
        """Signal the worker to stop gracefully."""
        logger.info("Stopping OCR Worker...")
        self.running = False


async def main() -> None:
    """Entry point for running the OCR worker as a standalone process."""
    import argparse

    parser = argparse.ArgumentParser(description="OCR Worker for Recall Pipeline")
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
        "--lang",
        default="eng",
        help="Tesseract language code (default: eng)",
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

    worker = OCRWorker(
        batch_size=args.batch_size,
        poll_interval=args.poll_interval,
        tesseract_lang=args.lang,
    )

    try:
        await worker.run()
    except KeyboardInterrupt:
        worker.stop()


if __name__ == "__main__":
    asyncio.run(main())
