#!/usr/bin/env python3
"""
PostgreSQL Database Migration Script for Mirix
Applies timezone awareness to timestamp columns.

For PostgreSQL, this is typically handled automatically by using TIMESTAMPTZ
column types. This script can be used to migrate existing TIMESTAMP columns
to TIMESTAMPTZ and ensure all values are interpreted as UTC.
"""

import os
import sys
from typing import Optional

try:
    import psycopg2
    from psycopg2 import sql
except ImportError:
    print("❌ psycopg2 is required for PostgreSQL migrations")
    print("Install with: pip install psycopg2-binary")
    sys.exit(1)


def migrate_timestamps_to_timestamptz(
    database_url: str,
    schema: str = "public",
    dry_run: bool = False,
    overwrite_nulls: bool = False,
):
    """
    Migrate TIMESTAMP columns to TIMESTAMPTZ, interpreting existing values as UTC.

    Args:
        database_url: PostgreSQL connection string
        schema: Database schema to process (default: public)
        dry_run: If True, only print what would be done without making changes
        overwrite_nulls: If True, set NULL timestamp values to current UTC time
    """
    print(f"{'[DRY RUN] ' if dry_run else ''}Starting PostgreSQL timezone migration")

    conn = psycopg2.connect(database_url)
    conn.autocommit = False

    try:
        cursor = conn.cursor()

        # Find all TIMESTAMP columns (without timezone) in the schema
        cursor.execute("""
            SELECT table_name, column_name, is_nullable
            FROM information_schema.columns
            WHERE table_schema = %s
              AND data_type = 'timestamp without time zone'
            ORDER BY table_name, column_name
        """, (schema,))

        timestamp_columns = cursor.fetchall()

        if not timestamp_columns:
            print("✓ No TIMESTAMP columns found - all columns may already be TIMESTAMPTZ")
            return

        print(f"Found {len(timestamp_columns)} TIMESTAMP columns to migrate:")
        for table, column, nullable in timestamp_columns:
            print(f"  - {table}.{column} (nullable: {nullable})")

        if dry_run:
            print("\n[DRY RUN] Would execute the following migrations:")

        for table_name, column_name, is_nullable in timestamp_columns:
            # Use sql.Identifier to safely quote identifiers
            table_id = sql.Identifier(table_name)
            column_id = sql.Identifier(column_name)

            # Step 1: Optionally fill NULL values
            if overwrite_nulls and is_nullable == 'YES':
                fill_nulls_query = sql.SQL("""
                    UPDATE {table} SET {column} = NOW() AT TIME ZONE 'UTC'
                    WHERE {column} IS NULL
                """).format(table=table_id, column=column_id)

                if dry_run:
                    print(f"  UPDATE {table_name} SET {column_name} = NOW() WHERE {column_name} IS NULL")
                else:
                    cursor.execute(fill_nulls_query)
                    if cursor.rowcount > 0:
                        print(f"  ✓ Filled {cursor.rowcount} NULL values in {table_name}.{column_name}")

            # Step 2: Convert column to TIMESTAMPTZ
            # First, we update the column type, treating existing values as UTC
            alter_query = sql.SQL("""
                ALTER TABLE {table}
                ALTER COLUMN {column} TYPE TIMESTAMPTZ
                USING {column} AT TIME ZONE 'UTC'
            """).format(table=table_id, column=column_id)

            if dry_run:
                print(f"  ALTER TABLE {table_name} ALTER COLUMN {column_name} TYPE TIMESTAMPTZ USING {column_name} AT TIME ZONE 'UTC'")
            else:
                try:
                    cursor.execute(alter_query)
                    print(f"  ✓ Converted {table_name}.{column_name} to TIMESTAMPTZ")
                except psycopg2.Error as e:
                    print(f"  ❌ Failed to convert {table_name}.{column_name}: {e}")
                    raise

        if not dry_run:
            conn.commit()
            print("\n✅ PostgreSQL timezone migration completed successfully!")
        else:
            print("\n[DRY RUN] No changes were made. Run without --dry-run to apply migrations.")

    except Exception as e:
        print(f"❌ Migration failed: {e}")
        conn.rollback()
        raise
    finally:
        conn.close()


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="Migrate PostgreSQL TIMESTAMP columns to TIMESTAMPTZ (UTC-aware)"
    )
    parser.add_argument(
        "--database-url",
        default=os.environ.get("DATABASE_URL"),
        help="PostgreSQL connection string (or set DATABASE_URL env var)",
    )
    parser.add_argument(
        "--schema",
        default="public",
        help="Database schema to process (default: public)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be done without making changes",
    )
    parser.add_argument(
        "--fill-nulls",
        action="store_true",
        help="Fill NULL timestamp values with current UTC time (default: leave NULLs unchanged)",
    )
    args = parser.parse_args()

    if not args.database_url:
        print("❌ Database URL required. Set DATABASE_URL env var or use --database-url")
        sys.exit(1)

    try:
        migrate_timestamps_to_timestamptz(
            database_url=args.database_url,
            schema=args.schema,
            dry_run=args.dry_run,
            overwrite_nulls=args.fill_nulls,
        )
    except Exception as e:
        print(f"❌ Migration failed: {e}")
        sys.exit(1)
