#!/usr/bin/env python3
"""
Database Migration Script for Mirix
Applies timezone awareness to datetime columns.

NOTE: This script is for SQLite databases only. For PostgreSQL, use
migrate_database_postgresql.sql or the timezone_migration_postgresql.py script.
"""

import sqlite3
from datetime import datetime, timezone


def migrate_database_inplace(db_path: str, overwrite_nulls: bool = False):
    """
    Migrate database in-place to add timezone awareness.

    Args:
        db_path: Path to the SQLite database file
        overwrite_nulls: If True, set NULL datetime values to current UTC time.
                         If False (default), leave NULL values unchanged.
    """

    print(f"Starting in-place migration of {db_path}")

    # Connect to the database
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA foreign_keys = OFF")  # Disable foreign keys during migration

    try:
        cursor = conn.cursor()

        # Get all tables
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table';")
        tables = [row[0] for row in cursor.fetchall()]

        for table_name in tables:
            print(f"Processing table: {table_name}")
            cursor.execute(f"PRAGMA table_info({table_name})")
            columns = cursor.fetchall()

            datetime_columns = [col[1] for col in columns if col[2] == 'DATETIME']

            if not datetime_columns:
                print(f"  No DATETIME columns found in {table_name}")
                continue

            for col_name in datetime_columns:
                print(f"  Converting column: {col_name}")

                # Only update NULL values if explicitly requested
                if overwrite_nulls:
                    conn.execute(
                        f"UPDATE {table_name} SET {col_name} = ? WHERE {col_name} IS NULL",
                        (datetime.now(timezone.utc).isoformat(),)
                    )
                    print(f"    - Filled NULL values with current UTC time")

                # Convert existing naive datetimes to UTC-aware
                cursor.execute(f"SELECT rowid, {col_name} FROM {table_name} WHERE {col_name} IS NOT NULL")
                rows = cursor.fetchall()
                converted_count = 0
                for rowid, dt_str in rows:
                    if dt_str:
                        try:
                            dt = datetime.fromisoformat(dt_str)
                            if dt.tzinfo is None:
                                dt = dt.replace(tzinfo=timezone.utc)
                                new_dt_str = dt.isoformat()
                                conn.execute(f"UPDATE {table_name} SET {col_name} = ? WHERE rowid = ?", (new_dt_str, rowid))
                                converted_count += 1
                        except ValueError:
                            print(f"    - Could not parse datetime string '{dt_str}' in table '{table_name}', column '{col_name}', rowid '{rowid}'")
                if converted_count > 0:
                    print(f"    - Converted {converted_count} naive datetimes to UTC")
            conn.commit()

        # Re-enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON")
        conn.commit()

        print("\n✅ In-place migration completed successfully!")
        print(f"Database: {db_path}")

    except Exception as e:
        print(f"❌ Migration failed: {e}")
        conn.rollback()
        raise
    finally:
        conn.close()


if __name__ == "__main__":
    import argparse
    import os
    import sys

    parser = argparse.ArgumentParser(description="Apply timezone awareness to a Mirix SQLite database.")
    parser.add_argument("db_path", nargs='?', default=os.path.expanduser("~/.mirix/sqlite.db"),
                        help="Path to the SQLite database file (defaults to ~/.mirix/sqlite.db)")
    parser.add_argument("--fill-nulls", action="store_true",
                        help="Fill NULL datetime values with current UTC time (default: leave NULLs unchanged)")
    args = parser.parse_args()

    if not os.path.exists(args.db_path):
        print(f"❌ Mirix database not found: {args.db_path}")
        sys.exit(1)

    try:
        migrate_database_inplace(args.db_path, overwrite_nulls=args.fill_nulls)
    except Exception as e:
        print(f"❌ Migration failed: {e}")
        sys.exit(1)
