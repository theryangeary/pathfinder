#!/bin/bash

# Daily PostgreSQL backup script with 7-day rotation
# Uses pg_dump to create backup and rclone to store to Tigris

set -euo pipefail

# Configuration
DB_HOST="${DB_HOST:-postgres}"
DB_USER="${DB_USER:-pathfinder}"
DB_NAME="${DB_NAME:-pathfinder}"
BACKUP_DIR="${BACKUP_DIR:-/tmp/backups}"
RCLONE_REMOTE="tigris-pathfinder-db-backup"
RCLONE_BUCKET="pathfinder-db-backup"
MAX_BACKUPS=7

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Generate backup filename with current date
DATE=$(date +%Y%m%d)
BACKUP_FILE_PREFIX=pathfinder_backup_
BACKUP_FILE="${BACKUP_FILE_PREFIX}${DATE}.sql"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_FILE"

echo "Starting backup process..."
echo "Date: $(date)"
echo "Backup file: $BACKUP_FILE"

# Create database backup using pg_dump
echo "Creating database backup..."
if ! pg_dump --host="$DB_HOST" --username="$DB_USER" --dbname="$DB_NAME" --no-password --clean --if-exists --create > "$BACKUP_PATH"; then
    echo "Error: Failed to create database backup"
    exit 1
fi

# Verify backup file was created and has content
if [[ ! -f "$BACKUP_PATH" ]] || [[ ! -s "$BACKUP_PATH" ]]; then
    echo "Error: Backup file is empty or doesn't exist"
    exit 1
fi

echo "Database backup created successfully: $BACKUP_PATH"
echo "Backup size: $(du -h "$BACKUP_PATH" | cut -f1)"

# Upload backup to Tigris using rclone
echo "Uploading backup to Tigris..."
if ! rclone copy "$BACKUP_PATH" "$RCLONE_REMOTE:$RCLONE_BUCKET/"; then
    echo "Error: Failed to upload backup to Tigris"
    exit 1
fi

echo "Backup uploaded successfully"

# Clean up local backup file
rm -f "$BACKUP_PATH"

# Implement 7-day rotation: remove oldest backups if more than MAX_BACKUPS
echo "Checking for old backups to remove..."
REMOTE_BACKUPS=$(rclone lsf "$RCLONE_REMOTE:$RCLONE_BUCKET/" | grep "^${BACKUP_FILE_PREFIX}" | sort)
BACKUP_COUNT=$(echo "$REMOTE_BACKUPS" | wc -l)

if [[ $BACKUP_COUNT -gt $MAX_BACKUPS ]]; then
    BACKUPS_TO_REMOVE=$((BACKUP_COUNT - MAX_BACKUPS))
    echo "Found $BACKUP_COUNT backups, removing oldest $BACKUPS_TO_REMOVE..."
    
    # Get the oldest backups to remove
    OLDEST_BACKUPS=$(echo "$REMOTE_BACKUPS" | head -n "$BACKUPS_TO_REMOVE")
    
    # Remove each old backup
    while IFS= read -r backup_file; do
        if [[ -n "$backup_file" ]]; then
            echo "Removing old backup: $backup_file"
            if ! rclone delete "$RCLONE_REMOTE:$RCLONE_BUCKET/$backup_file"; then
                echo "Warning: Failed to remove old backup: $backup_file"
            fi
        fi
    done <<< "$OLDEST_BACKUPS"
fi

echo "Backup process completed successfully"
echo "Current backups in storage:"
rclone lsf "$RCLONE_REMOTE:$RCLONE_BUCKET/" | grep "^${BACKUP_FILE_PREFIX}" | sort
