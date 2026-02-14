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
    print("Creating database tables...")
    Base.metadata.create_all(bind=engine)
    print("Database tables created successfully.")

if __name__ == "__main__":
    init_db()
