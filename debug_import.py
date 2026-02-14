import sys
import traceback

print(f"Python path: {sys.path}")

try:
    print("Attempting to import agents.processors.ocr_worker...")
    from agents.processors.ocr_worker import OCRWorker
    print("Success: agents.processors.ocr_worker imported.")
except Exception:
    traceback.print_exc()

try:
    print("Attempting to import agents.processors.vision_worker...")
    from agents.processors.vision_worker import VisionWorker
    print("Success: agents.processors.vision_worker imported.")
except Exception:
    traceback.print_exc()
