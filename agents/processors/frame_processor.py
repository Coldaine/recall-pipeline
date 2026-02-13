import logging
import time
from typing import List, Optional
from uuid import UUID

from sqlalchemy import text, select, update
from agents.database.connection import get_db
from agents.schemas.frame import Frame
from agents.settings import settings

logger = logging.getLogger(__name__)

class FrameProcessor:
    def __init__(self, batch_size: int = 10, poll_interval: int = 5):
        self.batch_size = batch_size
        self.poll_interval = poll_interval
        self.running = False

    def get_pending_vision_frames(self) -> List[Frame]:
        """
        Fetch frames that need vision processing.
        Uses SKIP LOCKED to allow multiple processors (if we ever scale).
        """
        db = next(get_db())
        frames = []
        try:
            # We select frames where vision_status = 0 (Pending)
            # And lock them to prevent other workers from picking them up
            # Note: This lock lasts until the transaction is committed/rolled back
            # But here we are just reading.
            # For true robust processing, we should mark them as 'processing' or hold the transaction.
            # Since we are doing a simple poll, we will fetch them and immediately update status to 'processing' (e.g. 99)
            # Or we just process them one by one.
            
            # Let's use a simple SELECT for now, assuming single worker.
            # If we wanted concurrent, we'd need:
            # SELECT * FROM frames WHERE vision_status = 0 LIMIT N FOR UPDATE SKIP LOCKED
            
            query = text("""
                SELECT * FROM frames 
                WHERE vision_status = 0 
                ORDER BY captured_at ASC 
                LIMIT :limit
            """)
            
            result = db.execute(query, {"limit": self.batch_size})
            rows = result.fetchall()
            
            # Convert to Pydantic models
            for row in rows:
                # Row is a tuple/named tuple. We need to map it.
                # Sqlalchemy returns a Row object which behaves like a dict or tuple
                frame_dict = row._mapping
                frames.append(Frame(**frame_dict))
                
        except Exception as e:
            logger.error(f"Error fetching pending frames: {e}")
            # Assuming db session might need rollback if error
            db.rollback()
        finally:
            db.close()
            
        return frames

    def mark_frame_processing(self, frame_ids: List[UUID]):
        # Optional: mark as 'processing' if we have that state
        pass

    def update_vision_summary(self, frame_id: UUID, summary: str):
        db = next(get_db())
        try:
            query = text("""
                UPDATE frames 
                SET vision_summary = :summary, vision_status = 1 
                WHERE id = :id
            """)
            db.execute(query, {"summary": summary, "id": frame_id})
            db.commit()
            logger.info(f"Updated summary for frame {frame_id}")
        except Exception as e:
            logger.error(f"Error updating frame {frame_id}: {e}")
            db.rollback()
        finally:
            db.close()

    def run_loop(self):
        """
        Main polling loop.
        """
        self.running = True
        logger.info("Starting Frame Processor Loop...")
        
        while self.running:
            try:
                frames = self.get_pending_vision_frames()
                
                if not frames:
                    time.sleep(self.poll_interval)
                    continue
                
                logger.info(f"Found {len(frames)} pending frames.")
                
                # Process frames
                # In a real implementation, this would call the Agent/LLM
                # For now, we'll just log
                for frame in frames:
                    self.process_frame(frame)
                    
            except KeyboardInterrupt:
                logger.info("Stopping Frame Processor...")
                self.running = False
            except Exception as e:
                logger.error(f"Error in processor loop: {e}")
                time.sleep(self.poll_interval)

    def process_frame(self, frame: Frame):
        """
        Process a single frame using the Vision Agent.
        """
        try:
            logger.info(f"Processing frame: {frame.id} - {frame.app_name}")
            
            # Lazy load agent to avoid init issues at startup if config is wrong
            if not hasattr(self, 'vision_agent'):
                from agents.agent.vision import VisionAgent
                self.vision_agent = VisionAgent()
            
            summary = self.vision_agent.summarize_frame(frame)
            
            if summary:
                logger.info(f"Generated summary: {summary[:50]}...")
                self.update_vision_summary(frame.id, summary)
            else:
                logger.warning(f"No summary generated for frame {frame.id}")
                # Mark as failed or skipped? For now, just leave as processed but empty?
                # Or status = 2 (Failed)
                self.update_vision_status(frame.id, 2) 
                
        except Exception as e:
            logger.error(f"Error processing frame {frame.id}: {e}")
            self.update_vision_status(frame.id, 2)

    def update_vision_status(self, frame_id: UUID, status: int):
        db = next(get_db())
        try:
            query = text("UPDATE frames SET vision_status = :status WHERE id = :id")
            db.execute(query, {"status": status, "id": frame_id})
            db.commit()
        except Exception as e:
            logger.error(f"Error updating status for frame {frame_id}: {e}")
            db.rollback()
        finally:
            db.close()

if __name__ == "__main__":
    # Configure logging
    logging.basicConfig(level=logging.INFO)
    
    processor = FrameProcessor()
    processor.run_loop()
