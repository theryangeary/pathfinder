#!/bin/bash

# Setup rclone configuration for Tigris storage
# Uses AWS-compatible environment variables

set -euo pipefail

# Check if required environment variables are set
if [[ -z "${AWS_ACCESS_KEY_ID:-}" ]]; then
    echo "Error: AWS_ACCESS_KEY_ID environment variable is not set"
    exit 1
fi

if [[ -z "${AWS_SECRET_ACCESS_KEY:-}" ]]; then
    echo "Error: AWS_SECRET_ACCESS_KEY environment variable is not set"
    exit 1
fi

# Set defaults for optional variables
AWS_ENDPOINT_URL_S3="${AWS_ENDPOINT_URL_S3:-https://fly.storage.tigris.dev}"
AWS_REGION="${AWS_REGION:-auto}"

# Check if rclone remote already exists
if rclone listremotes | grep -q "^${PATHFINDER_TIGRIS_POSTGRES_BACKUP}:$"; then
    echo "rclone remote '${PATHFINDER_TIGRIS_POSTGRES_BACKUP}' already exists"
    exit 0
fi

echo "Creating rclone remote '${PATHFINDER_TIGRIS_POSTGRES_BACKUP}' for Tigris storage..."

# Create rclone configuration for S3-compatible storage
rclone config create "$PATHFINDER_TIGRIS_POSTGRES_BACKUP" s3 \
    provider=Other \
    access_key_id="$AWS_ACCESS_KEY_ID" \
    secret_access_key="$AWS_SECRET_ACCESS_KEY" \
    endpoint="$AWS_ENDPOINT_URL_S3" \
    region="$AWS_REGION" \
    acl=private

echo "rclone remote '${PATHFINDER_TIGRIS_POSTGRES_BACKUP}' created successfully"

# Test the connection
echo "Testing connection to Tigris storage..."
if rclone lsf "${PATHFINDER_TIGRIS_POSTGRES_BACKUP}:" > /dev/null 2>&1; then
    echo "Connection test successful"
else
    echo "Warning: Connection test failed - check your credentials and endpoint"
    exit 1
fi
