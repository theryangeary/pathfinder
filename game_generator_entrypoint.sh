#!/bin/sh

if [ "$RUN_MODE" = "cron" ]; then
    echo "Setting up cron mode..."
    
    # Create cron job to run game generator daily at 2 AM UTC
    echo "0 2 * * * cd /app && ./game-generator >> /var/log/game-generator.log 2>&1" > /etc/crontabs/root
    
    # Create log file
    touch /var/log/game-generator.log

    ./game-generator
    
    echo "Starting crond in foreground..."
    exec crond -f
else
    echo "Running game generator once..."
    exec ./game-generator
fi
