import pytest
import asyncio
import os
import asyncpg
from typing import AsyncGenerator
from agents.database.connection import get_db_connection

# Use a default test database URL if not provided
# WARNING: This defaults to the local postgres instance. 
# Ideally, use a separate test database to avoid data loss.
TEST_DB_URL = os.getenv("TEST_DATABASE_URL", "postgresql://postgres:postgres@localhost:5432/recall_test_db")

@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for each test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()

@pytest.fixture(scope="session")
async def db_pool():
    """
    Session-scoped database connection pool.
    """
    # Overwrite the DATABASE_URL environment variable for the duration of tests
    # to ensure the application code uses the test database.
    os.environ["DATABASE_URL"] = TEST_DB_URL
    
    try:
        pool = await asyncpg.create_pool(TEST_DB_URL)
        yield pool
        await pool.close()
    except Exception as e:
        pytest.fail(f"Could not connect to test database at {TEST_DB_URL}: {e}")

@pytest.fixture(scope="function")
async def db_conn(db_pool) -> AsyncGenerator[asyncpg.Connection, None]:
    """
    Function-scoped database connection that cleans up after itself.
    Wraps the test in a transaction and rolls it back at the end.
    """
    async with db_pool.acquire() as conn:
        tr = conn.transaction()
        await tr.start()
        
        yield conn
        
        await tr.rollback()

@pytest.fixture(scope="function")
async def clean_db(db_pool):
    """
    Fixture to truncate relevant tables before specific tests if transaction rollback isn't enough
    (though transaction rollback is preferred).
    """
    async with db_pool.acquire() as conn:
        await conn.execute("TRUNCATE TABLE frames, ocr_text RESTART IDENTITY CASCADE")

