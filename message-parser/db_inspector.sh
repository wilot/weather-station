#!/bin/bash

# SQLite Database Inspector - Last 10 Records
# Usage: ./db_inspector.sh [database_file] [table_name]

# Default values
DB_FILE="${1:-database.db}"
TABLE_NAME="${2:-weather_data}"

# Check if database file exists
if [ ! -f "$DB_FILE" ]; then
    echo "Error: Database file '$DB_FILE' not found!"
    echo "Usage: $0 [database_file] [table_name]"
    exit 1
fi

# Check if table exists
TABLE_EXISTS=$(sqlite3 "$DB_FILE" "SELECT name FROM sqlite_master WHERE type='table' AND name='$TABLE_NAME';" 2>/dev/null)

if [ -z "$TABLE_EXISTS" ]; then
    echo "Error: Table '$TABLE_NAME' not found in database!"
    echo "Available tables:"
    sqlite3 "$DB_FILE" ".tables"
    exit 1
fi

echo "=== Last 10 Records from '$TABLE_NAME' ==="
echo "Database: $DB_FILE"
echo "==========================================="

# Execute SQLite query with formatting
sqlite3 "$DB_FILE" << EOF
.mode column
.headers on
.width 5 15 20 15 12 25
SELECT * FROM $TABLE_NAME ORDER BY ReceivedTime DESC LIMIT 10;
EOF
