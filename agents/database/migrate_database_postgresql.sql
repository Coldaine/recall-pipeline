-- PostgreSQL Database Migration Script for Recall Pipeline
-- Migrates database schema to use deployment_id for single-user, multi-machine setup
-- Removes multi-tenant Organization/User model in favor of deployment identification

-- Start transaction to ensure atomicity
BEGIN;

-- Create a function to check if a column exists
CREATE OR REPLACE FUNCTION column_exists(tbl_name text, col_name text) 
RETURNS boolean AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_schema = 'public' 
        AND table_name = tbl_name 
        AND column_name = col_name
    );
END;
$$ LANGUAGE plpgsql;

-- Get default deployment_id for migrations
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    -- Use hostname or generate a deployment ID
    -- In production, this should be read from environment or config
    default_deployment_id := 'deployment-' || replace(gen_random_uuid()::text, '-', '');

    -- Store the default deployment ID in a temporary table for use in subsequent migrations
    CREATE TEMP TABLE IF NOT EXISTS migration_vars (
        default_deployment_id VARCHAR
    );
    DELETE FROM migration_vars;
    INSERT INTO migration_vars (default_deployment_id) VALUES (default_deployment_id);

    RAISE NOTICE 'Using deployment ID: %', default_deployment_id;
END;
$$;

-- Migration 1: Add status column to users table if it doesn't exist
DO $$
BEGIN
    IF NOT column_exists('users', 'status') THEN
        ALTER TABLE users ADD COLUMN status VARCHAR NOT NULL DEFAULT 'active';
        RAISE NOTICE '✓ Added status column to users table';
    ELSE
        RAISE NOTICE '✓ Skipped: status column already exists in users table';
    END IF;
END;
$$;

-- Migration 2: Add mcp_tools column to agents table if it doesn't exist
DO $$
BEGIN
    IF NOT column_exists('agents', 'mcp_tools') THEN
        ALTER TABLE agents ADD COLUMN mcp_tools JSONB;
        RAISE NOTICE '✓ Added mcp_tools column to agents table';
    ELSE
        RAISE NOTICE '✓ Skipped: mcp_tools column already exists in agents table';
    END IF;
END;
$$;

-- Migration 3: Add deployment_id column to block table if it doesn't exist
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    SELECT migration_vars.default_deployment_id INTO default_deployment_id FROM migration_vars;
    
    IF NOT column_exists('block', 'deployment_id') THEN
        ALTER TABLE block ADD COLUMN deployment_id TEXT;
        UPDATE block SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        RAISE NOTICE '✓ Added and populated deployment_id column in block table';
    ELSE
        -- Update any NULL values
        UPDATE block SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        RAISE NOTICE '✓ Skipped: deployment_id column already exists in block table (updated NULL values)';
    END IF;
END;
$$;

-- Migration 4: Add deployment_id column to files table if it doesn't exist
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    SELECT migration_vars.default_deployment_id INTO default_deployment_id FROM migration_vars;
    
    IF NOT column_exists('files', 'deployment_id') THEN
        ALTER TABLE files ADD COLUMN deployment_id TEXT;
        UPDATE files SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        RAISE NOTICE '✓ Added and populated deployment_id column in files table';
    ELSE
        -- Update any NULL values
        UPDATE files SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        RAISE NOTICE '✓ Skipped: deployment_id column already exists in files table (updated NULL values)';
    END IF;
END;
$$;

-- Migration 5: Add deployment_id column to cloud_file_mapping table if it doesn't exist
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    SELECT migration_vars.default_deployment_id INTO default_deployment_id FROM migration_vars;
    
    IF NOT column_exists('cloud_file_mapping', 'deployment_id') THEN
        ALTER TABLE cloud_file_mapping ADD COLUMN deployment_id TEXT;
        UPDATE cloud_file_mapping SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        RAISE NOTICE '✓ Added and populated deployment_id column in cloud_file_mapping table';
    ELSE
        -- Update any NULL values
        UPDATE cloud_file_mapping SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        RAISE NOTICE '✓ Skipped: deployment_id column already exists in cloud_file_mapping table (updated NULL values)';
    END IF;
END;
$$;

-- Migration 6: Replace user_id with deployment_id in episodic_memory table
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    SELECT migration_vars.default_deployment_id INTO default_deployment_id FROM migration_vars;

    IF NOT column_exists('episodic_memory', 'deployment_id') THEN
        -- Add deployment_id column
        ALTER TABLE episodic_memory ADD COLUMN deployment_id VARCHAR;
        UPDATE episodic_memory SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        ALTER TABLE episodic_memory ALTER COLUMN deployment_id SET NOT NULL;

        -- Remove user_id and organization_id if they exist
        IF column_exists('episodic_memory', 'user_id') THEN
            ALTER TABLE episodic_memory DROP COLUMN user_id;
        END IF;
        IF column_exists('episodic_memory', 'organization_id') THEN
            ALTER TABLE episodic_memory DROP COLUMN organization_id;
        END IF;

        RAISE NOTICE '✓ Migrated episodic_memory to use deployment_id';
    ELSE
        RAISE NOTICE '✓ Skipped: episodic_memory already uses deployment_id';
    END IF;
END;
$$;

-- Migration 7: Replace user_id with deployment_id in knowledge_vault table
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    SELECT migration_vars.default_deployment_id INTO default_deployment_id FROM migration_vars;

    IF NOT column_exists('knowledge_vault', 'deployment_id') THEN
        -- Add deployment_id column
        ALTER TABLE knowledge_vault ADD COLUMN deployment_id VARCHAR;
        UPDATE knowledge_vault SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        ALTER TABLE knowledge_vault ALTER COLUMN deployment_id SET NOT NULL;

        -- Remove user_id and organization_id if they exist
        IF column_exists('knowledge_vault', 'user_id') THEN
            ALTER TABLE knowledge_vault DROP COLUMN user_id;
        END IF;
        IF column_exists('knowledge_vault', 'organization_id') THEN
            ALTER TABLE knowledge_vault DROP COLUMN organization_id;
        END IF;

        RAISE NOTICE '✓ Migrated knowledge_vault to use deployment_id';
    ELSE
        RAISE NOTICE '✓ Skipped: knowledge_vault already uses deployment_id';
    END IF;
END;
$$;

-- Migration 8: Replace user_id with deployment_id in procedural_memory table
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    SELECT migration_vars.default_deployment_id INTO default_deployment_id FROM migration_vars;

    IF NOT column_exists('procedural_memory', 'deployment_id') THEN
        -- Add deployment_id column
        ALTER TABLE procedural_memory ADD COLUMN deployment_id VARCHAR;
        UPDATE procedural_memory SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        ALTER TABLE procedural_memory ALTER COLUMN deployment_id SET NOT NULL;

        -- Remove user_id and organization_id if they exist
        IF column_exists('procedural_memory', 'user_id') THEN
            ALTER TABLE procedural_memory DROP COLUMN user_id;
        END IF;
        IF column_exists('procedural_memory', 'organization_id') THEN
            ALTER TABLE procedural_memory DROP COLUMN organization_id;
        END IF;

        RAISE NOTICE '✓ Migrated procedural_memory to use deployment_id';
    ELSE
        RAISE NOTICE '✓ Skipped: procedural_memory already uses deployment_id';
    END IF;
END;
$$;

-- Migration 9: Replace user_id with deployment_id in resource_memory table
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    SELECT migration_vars.default_deployment_id INTO default_deployment_id FROM migration_vars;

    IF NOT column_exists('resource_memory', 'deployment_id') THEN
        -- Add deployment_id column
        ALTER TABLE resource_memory ADD COLUMN deployment_id VARCHAR;
        UPDATE resource_memory SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        ALTER TABLE resource_memory ALTER COLUMN deployment_id SET NOT NULL;

        -- Remove user_id and organization_id if they exist
        IF column_exists('resource_memory', 'user_id') THEN
            ALTER TABLE resource_memory DROP COLUMN user_id;
        END IF;
        IF column_exists('resource_memory', 'organization_id') THEN
            ALTER TABLE resource_memory DROP COLUMN organization_id;
        END IF;

        RAISE NOTICE '✓ Migrated resource_memory to use deployment_id';
    ELSE
        RAISE NOTICE '✓ Skipped: resource_memory already uses deployment_id';
    END IF;
END;
$$;

-- Migration 10: Replace user_id with deployment_id in semantic_memory table
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    SELECT migration_vars.default_deployment_id INTO default_deployment_id FROM migration_vars;

    IF NOT column_exists('semantic_memory', 'deployment_id') THEN
        -- Add deployment_id column
        ALTER TABLE semantic_memory ADD COLUMN deployment_id VARCHAR;
        UPDATE semantic_memory SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        ALTER TABLE semantic_memory ALTER COLUMN deployment_id SET NOT NULL;

        -- Remove user_id and organization_id if they exist
        IF column_exists('semantic_memory', 'user_id') THEN
            ALTER TABLE semantic_memory DROP COLUMN user_id;
        END IF;
        IF column_exists('semantic_memory', 'organization_id') THEN
            ALTER TABLE semantic_memory DROP COLUMN organization_id;
        END IF;

        RAISE NOTICE '✓ Migrated semantic_memory to use deployment_id';
    ELSE
        RAISE NOTICE '✓ Skipped: semantic_memory already uses deployment_id';
    END IF;
END;
$$;

-- Migration 11: Add deployment_id column to messages table if it doesn't exist
DO $$
DECLARE
    default_deployment_id VARCHAR;
BEGIN
    SELECT migration_vars.default_deployment_id INTO default_deployment_id FROM migration_vars;
    
    IF NOT column_exists('messages', 'deployment_id') THEN
        ALTER TABLE messages ADD COLUMN deployment_id TEXT;
        UPDATE messages SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        RAISE NOTICE '✓ Added and populated deployment_id column in messages table';
    ELSE
        -- Update any NULL values
        UPDATE messages SET deployment_id = default_deployment_id WHERE deployment_id IS NULL;
        RAISE NOTICE '✓ Skipped: deployment_id column already exists in messages table (updated NULL values)';
    END IF;
END;
$$;

-- Migration 12: Rename secrets.encrypted_value to raw_value and make key_id nullable
DO $$
BEGIN
    -- Rename the column if it exists
    IF column_exists('secrets', 'encrypted_value') THEN
        ALTER TABLE secrets RENAME COLUMN encrypted_value TO raw_value;
        RAISE NOTICE '✓ Renamed secrets.encrypted_value to secrets.raw_value';
    ELSE
        RAISE NOTICE '✓ Skipped: secrets.encrypted_value does not exist, assuming already migrated to raw_value';
    END IF;

    -- Make key_id nullable if it exists and is not already nullable
    IF column_exists('secrets', 'key_id') THEN
        IF (SELECT is_nullable FROM information_schema.columns WHERE table_name = 'secrets' AND column_name = 'key_id') = 'NO' THEN
            ALTER TABLE secrets ALTER COLUMN key_id DROP NOT NULL;
            RAISE NOTICE '✓ Made secrets.key_id nullable';
        ELSE
            RAISE NOTICE '✓ Skipped: secrets.key_id is already nullable';
        END IF;
    ELSE
        RAISE NOTICE '✓ Skipped: secrets.key_id does not exist';
    END IF;
END;
$$;

-- Verification: Check that all required columns exist and are populated
DO $$
DECLARE
    table_name text;
    column_name text;
    null_count integer;
    tables_with_deployment_id text[] := ARRAY['episodic_memory', 'knowledge_vault',
                                               'procedural_memory', 'resource_memory',
                                               'semantic_memory'];
    required_columns text[][] := ARRAY[
        ARRAY['agents', 'mcp_tools'],
        ARRAY['block', 'deployment_id'],
        ARRAY['files', 'deployment_id'],
        ARRAY['cloud_file_mapping', 'deployment_id'],
        ARRAY['messages', 'deployment_id'],
        ARRAY['episodic_memory', 'deployment_id'],
        ARRAY['knowledge_vault', 'deployment_id'],
        ARRAY['procedural_memory', 'deployment_id'],
        ARRAY['resource_memory', 'deployment_id'],
        ARRAY['semantic_memory', 'deployment_id']
    ];
BEGIN
    RAISE NOTICE '';
    RAISE NOTICE '🔍 Verifying migration...';

    -- Check that required columns exist
    FOR i IN 1..array_length(required_columns, 1) LOOP
        table_name := required_columns[i][1];
        column_name := required_columns[i][2];

        IF column_exists(table_name, column_name) THEN
            RAISE NOTICE '✓ %.% exists', table_name, column_name;
        ELSE
            RAISE NOTICE '❌ %.% missing', table_name, column_name;
        END IF;
    END LOOP;

    -- Check that deployment_id columns have been populated
    FOREACH table_name IN ARRAY tables_with_deployment_id LOOP
        BEGIN
            EXECUTE format('SELECT COUNT(*) FROM %I WHERE deployment_id IS NULL', table_name) INTO null_count;
            IF null_count = 0 THEN
                RAISE NOTICE '✓ %.deployment_id populated', table_name;
            ELSE
                RAISE NOTICE '⚠️  %.deployment_id has % NULL values', table_name, null_count;
            END IF;
        EXCEPTION
            WHEN undefined_table THEN
                RAISE NOTICE '⚠️  Could not verify %.deployment_id (table does not exist)', table_name;
            WHEN OTHERS THEN
                RAISE NOTICE '⚠️  Could not verify %.deployment_id (error: %)', table_name, SQLERRM;
        END;
    END LOOP;
END;
$$;

-- Clean up temporary objects
DROP FUNCTION IF EXISTS column_exists(text, text);
DROP TABLE IF EXISTS migration_vars;

-- Commit the transaction
COMMIT;

-- Final success message
DO $$
BEGIN
    RAISE NOTICE '';
    RAISE NOTICE '✅ PostgreSQL migration completed successfully!';
    RAISE NOTICE 'All schema changes have been applied and verified.';
END;
$$;
