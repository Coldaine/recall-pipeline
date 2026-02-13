import asyncio
import logging
from contextlib import asynccontextmanager
from typing import AsyncIterator, Generator, Optional

import asyncpg
from sqlalchemy import create_engine, text
from sqlalchemy.engine.url import make_url
from sqlalchemy.orm import Session, sessionmaker

from agents.settings import settings

logger = logging.getLogger(__name__)

# Create engine
# Use settings.agents_pg_uri which constructs the URI from env vars or defaults
# We use pool_pre_ping=True to handle disconnected sessions
engine = create_engine(
    settings.agents_pg_uri,
    pool_size=settings.pg_pool_size,
    max_overflow=settings.pg_max_overflow,
    pool_timeout=settings.pg_pool_timeout,
    pool_recycle=settings.pg_pool_recycle,
    pool_pre_ping=True,
    echo=settings.pg_echo,
)

SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)


def get_db() -> Generator[Session, None, None]:
    """
    Dependency for getting a database session.
    Yields a Session object and ensures it's closed after use.
    """
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()


def verify_connection() -> bool:
    """
    Verifies that the database connection is working.
    """
    try:
        with engine.connect() as connection:
            # Simple query to check connection
            from sqlalchemy import text
            connection.execute(text("SELECT 1"))
        return True
    except Exception as e:
        logger.error(f"Database connection failed: {e}")
        return False


def _build_async_dsn() -> str:
    """Convert the configured SQLAlchemy URI into a DSN compatible with asyncpg."""
    url = make_url(settings.agents_pg_uri)
    base_driver = url.drivername.split("+", 1)[0]
    async_url = url.set(drivername=base_driver)
    return async_url.render_as_string(hide_password=False)


_ASYNC_DSN = _build_async_dsn()
_async_pool: Optional[asyncpg.Pool] = None
_pool_lock = asyncio.Lock()


async def _get_async_pool() -> asyncpg.Pool:
    global _async_pool
    if _async_pool is None:
        async with _pool_lock:
            if _async_pool is None:
                parsed = make_url(settings.agents_pg_uri)
                logger.info(
                    "Initializing asyncpg pool (host=%s db=%s)",
                    parsed.host,
                    parsed.database,
                )
                _async_pool = await asyncpg.create_pool(
                    dsn=_ASYNC_DSN,
                    min_size=1,
                    max_size=settings.pg_pool_size,
                    timeout=settings.pg_pool_timeout,
                    command_timeout=settings.pg_pool_timeout,
                )
    return _async_pool


@asynccontextmanager
async def get_db_connection() -> AsyncIterator[asyncpg.Connection]:
    """Yield an asyncpg connection from the shared pool."""
    pool = await _get_async_pool()
    connection = await pool.acquire()
    try:
        yield connection
    finally:
        await pool.release(connection)


async def verify_async_connection() -> bool:
    """Verify the async connection path by running a lightweight query."""
    try:
        async with get_db_connection() as conn:
            await conn.execute("SELECT 1")
        return True
    except Exception as exc:
        logger.error("Async database connection failed: %s", exc)
        return False
