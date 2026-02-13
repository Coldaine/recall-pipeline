import asyncio
import os
import sys
from typing import Optional

from sqlalchemy import text
from sqlalchemy.engine.url import make_url

# Add the parent directory to sys.path to allow imports from agents
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from agents.database.connection import (
    get_db,
    get_db_connection,
    verify_async_connection,
    verify_connection,
)
from agents.settings import settings


def _safe_pg_uri(uri: str) -> str:
    url = make_url(uri)
    masked = url.set(password="***") if url.password else url
    return masked.render_as_string(hide_password=False)


async def fetch_latest_frame() -> Optional[dict]:
    async with get_db_connection() as conn:
        row = await conn.fetchrow(
            """
            SELECT id,
                   captured_at,
                   window_title,
                   app_name,
                   phash,
                   phash_prefix,
                   ocr_status,
                   vision_status,
                   ocr_text
            FROM frames
            ORDER BY captured_at DESC
            LIMIT 1
            """
        )
        if row:
            return dict(row)
        return None


async def main() -> None:
    print(f"Testing connection to: {_safe_pg_uri(settings.agents_pg_uri)}")

    sync_ok = verify_connection()
    print("[OK] Sync engine connected." if sync_ok else "[FAIL] Sync engine failed.")

    async_ok = await verify_async_connection()
    print("[OK] Async pool connected." if async_ok else "[FAIL] Async pool failed.")

    if not (sync_ok and async_ok):
        return

    # List tables via sync session for quick schema sanity check
    try:
        db = next(get_db())
        try:
            result = db.execute(
                text("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'")
            )
            tables = [row[0] for row in result]
            print(f"Tables in database: {tables}")
        finally:
            db.close()
    except Exception as exc:
        print(f"[WARN] Unable to list tables: {exc}")

    try:
        latest = await fetch_latest_frame()
        if latest:
            print("[OK] Latest frame:")
            print(
                f"  id={latest['id']} captured_at={latest['captured_at']}\n"
                f"  window_title={latest['window_title']} app_name={latest['app_name']}\n"
                f"  phash={latest['phash']} prefix={latest['phash_prefix']}\n"
                f"  ocr_status={latest['ocr_status']} vision_status={latest['vision_status']}\n"
                f"  ocr_text={(latest['ocr_text'] or '')[:120]}"
            )
        else:
            print("[WARN] No frames found. Has the Rust capture pipeline inserted data yet?")
    except Exception as exc:
        print(f"[FAIL] Failed to query latest frame: {exc}")


if __name__ == "__main__":
    asyncio.run(main())
