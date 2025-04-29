#!/bin/bash

# Settings
DB_NAME="test_db"
DB_USER="postgres"
DB_HOST="127.0.0.1"
DB_PORT="5432"

# Prompt for password
echo "Enter PostgreSQL password for user $DB_USER:"
read -s PGPASSWORD

# Export password for psql to use
export PGPASSWORD

# Run the cleanup
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 <<SQL
DO \$\$
DECLARE
    r RECORD;
BEGIN
    -- Disable triggers temporarily
    EXECUTE 'SET session_replication_role = replica';

    -- Drop all tables in public schema
    FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = 'public') LOOP
        EXECUTE 'DROP TABLE IF EXISTS public.' || quote_ident(r.tablename) || ' CASCADE';
    END LOOP;

    -- Restore triggers
    EXECUTE 'SET session_replication_role = DEFAULT';
END
\$\$;
SQL

# Cleanup environment
unset PGPASSWORD

echo "✅ All tables in database '$DB_NAME' have been dropped."
