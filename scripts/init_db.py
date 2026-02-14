"""
Database initialization script for Recall Pipeline.

Creates all database tables based on SQLAlchemy ORM models.

CRITICAL: This script is UNTESTED. Before production use:
  - Verify database schema matches ORM models
  - Test with clean PostgreSQL instance
  - Test idempotency (running twice should not fail)
  - Verify migrations strategy (is this for dev only?)
  - Add rollback capability

TODO: Add unit/integration tests for this script:
  - Test table creation with empty PostgreSQL
  - Test that all expected tables are created
  - Test schema column types and constraints match ORM
  - Test idempotency (running script twice is safe)
  - Test with invalid database connection (graceful error)
  - Test with restricted user permissions
  - Verify indexes are created correctly
  - Test foreign key constraints are established
  - Test default values are applied
  - Test enum types are created correctly

TODO: Add schema validation:
  - After init, query schema and verify tables exist
  - Compare actual schema with ORM definitions
  - Check all columns, types, and constraints
  - Verify indexes match ORM index definitions
  - Verify foreign keys are properly set up

TODO: Consider migration strategy:
  - Is this for development only?
  - Do we need migration versioning (Alembic)?
  - How do we handle schema updates?
  - How do we rollback failed migrations?
"""

import sys
import os

# Add project root to path
sys.path.append(os.getcwd())

from agents.database.connection import engine
from agents.orm.base import Base

# Import all models to ensure they are registered with Base.metadata
import agents.orm
from agents.orm import episodic_memory
from agents.orm import semantic_memory
from agents.orm import procedural_memory
from agents.orm import knowledge_vault
from agents.orm import resource_memory
from agents.orm import cloud_file_mapping

def init_db():
    """
    Initialize database by creating all tables defined in ORM models.
    
    WARNING: This is a simple create_all() call. It:
    - Does NOT handle migrations
    - Does NOT validate schema
    - Does NOT rollback on failure
    
    TODO: Consider using proper migration tool (Alembic) for production.
    """
    print("Creating database tables...")
    Base.metadata.create_all(bind=engine)
    print("Database tables created successfully.")

if __name__ == "__main__":
    init_db()
