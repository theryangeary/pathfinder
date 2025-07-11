#!/bin/bash

# Restore PostgreSQL backup from Tigris storage
# Downloads backup using rclone and restores using psql

set -euo pipefail

# Configuration with defaults
DB_HOST="${DB_HOST:-postgres}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-pathfinder}"
DB_NAME="${DB_NAME:-pathfinder}"
BACKUP_DIR="${BACKUP_DIR:-/tmp/backups}"
RCLONE_REMOTE="${PATHFINDER_TIGRIS_POSTGRES_BACKUP}"
RCLONE_BUCKET="pathfinder-db-backup"
BACKUP_FILE_PREFIX="pathfinder_backup_"

# Check if required environment variables are set
if [[ -z "${PATHFINDER_TIGRIS_POSTGRES_BACKUP:-}" ]]; then
    echo "Error: PATHFINDER_TIGRIS_POSTGRES_BACKUP environment variable is not set"
    exit 1
fi

# Function to show usage
usage() {
    echo "Usage: $0 [DATE] [OPTIONS]"
    echo ""
    echo "Arguments:"
    echo "  DATE                 Backup date in YYYYMMDD format (default: most recent)"
    echo ""
    echo "Options:"
    echo "  --host HOST          Database host (default: $DB_HOST)"
    echo "  --port PORT          Database port (default: $DB_PORT)"
    echo "  --user USER          Database user (default: $DB_USER)"
    echo "  --dbname DBNAME      Database name (default: $DB_NAME)"
    echo "  --help               Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  PGPASSWORD           Database password (required)"
    echo ""
    echo "Examples:"
    echo "  $0                   # Restore most recent backup"
    echo "  $0 20240315          # Restore backup from March 15, 2024"
    echo "  $0 --host localhost --port 5433 --user myuser --dbname mydb"
    exit 1
}

# Parse command line arguments
BACKUP_DATE=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --host)
            DB_HOST="$2"
            shift 2
            ;;
        --port)
            DB_PORT="$2"
            shift 2
            ;;
        --user)
            DB_USER="$2"
            shift 2
            ;;
        --dbname)
            DB_NAME="$2"
            shift 2
            ;;
        --help)
            usage
            ;;
        -*)
            echo "Unknown option: $1"
            usage
            ;;
        *)
            if [[ -z "$BACKUP_DATE" ]]; then
                BACKUP_DATE="$1"
            else
                echo "Unexpected argument: $1"
                usage
            fi
            shift
            ;;
    esac
done

# Check if PGPASSWORD is set
if [[ -z "${PGPASSWORD:-}" ]]; then
    echo "Error: PGPASSWORD environment variable must be set"
    exit 1
fi

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# If no date specified, find the most recent backup
if [[ -z "$BACKUP_DATE" ]]; then
    echo "Finding most recent backup..."
    REMOTE_BACKUPS=$(rclone lsf "$RCLONE_REMOTE:$RCLONE_BUCKET/" | grep "^${BACKUP_FILE_PREFIX}" | sort -r)
    
    if [[ -z "$REMOTE_BACKUPS" ]]; then
        echo "Error: No backups found in storage"
        exit 1
    fi
    
    # Get the most recent backup filename
    LATEST_BACKUP=$(echo "$REMOTE_BACKUPS" | head -n 1)
    BACKUP_DATE=$(echo "$LATEST_BACKUP" | sed "s/^${BACKUP_FILE_PREFIX}//" | sed 's/\.sql$//')
    
    echo "Most recent backup date: $BACKUP_DATE"
fi

# Validate date format
if [[ ! "$BACKUP_DATE" =~ ^[0-9]{8}$ ]]; then
    echo "Error: Date must be in YYYYMMDD format"
    exit 1
fi

# Construct backup filename
BACKUP_FILE="${BACKUP_FILE_PREFIX}${BACKUP_DATE}.sql"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_FILE"

# Ensure rclone is configured
if ! ./setup_rclone.sh; then
    echo "Error: Failed to setup rclone configuration"
    exit 1
fi

echo "Starting restore process..."
echo "Date: $(date)"
echo "Backup to restore: $BACKUP_FILE"
echo "Database: $DB_NAME@$DB_HOST:$DB_PORT (user: $DB_USER)"

# Download backup from Tigris
echo "Downloading backup from Tigris..."
if ! rclone copy "$RCLONE_REMOTE:$RCLONE_BUCKET/$BACKUP_FILE" "$BACKUP_DIR/"; then
    echo "Error: Failed to download backup file: $BACKUP_FILE"
    echo "Available backups:"
    rclone lsf "$RCLONE_REMOTE:$RCLONE_BUCKET/" | grep "^${BACKUP_FILE_PREFIX}" | sort -r
    exit 1
fi

# Verify backup file was downloaded and has content
if [[ ! -f "$BACKUP_PATH" ]] || [[ ! -s "$BACKUP_PATH" ]]; then
    echo "Error: Downloaded backup file is empty or doesn't exist"
    exit 1
fi

echo "Backup downloaded successfully: $BACKUP_PATH"
echo "Backup size: $(du -h "$BACKUP_PATH" | cut -f1)"

# Restore database using psql
echo "Restoring database..."
echo "WARNING: This will drop and recreate the database!"
read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Restore cancelled"
    rm -f "$BACKUP_PATH"
    exit 0
fi

if ! psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -f "$BACKUP_PATH"; then
    echo "Error: Failed to restore database"
    rm -f "$BACKUP_PATH"
    exit 1
fi

echo "Database restored successfully"

# Clean up downloaded backup file
rm -f "$BACKUP_PATH"

echo "Restore process completed successfully"