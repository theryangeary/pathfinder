# Fly.io Deployment Guide

This guide covers deploying the Pathfinder game to Fly.io with PostgreSQL.

## Prerequisites

1. Install the Fly CLI: https://fly.io/docs/flyctl/install/
2. Create a Fly.io account and login:
   ```bash
   flyctl auth signup
   flyctl auth login
   ```

## Initial Setup

1. **Create the Fly.io app:**
   ```bash
   flyctl apps create pathfinder-game
   ```

2. **Create a PostgreSQL database:**
   ```bash
   flyctl postgres create --name pathfinder-db --region iad
   ```

3. **Attach the database to your app:**
   ```bash
   flyctl postgres attach --app pathfinder-game pathfinder-db
   ```

4. **Set environment variables:**
   ```bash
   flyctl secrets set \
     RUST_LOG=info \
     SERVER_HOST=0.0.0.0 \
     HTTP_PORT=8080 \
     CORS_ALLOWED_ORIGINS=https://pathfinder-game.fly.dev \
     REFERER_VALIDATION_ENABLED=true \
     SESSION_TIMEOUT_MINUTES=1440 \
     RATE_LIMIT_PER_MINUTE=100
   ```

## Deployment

1. **Build and push the image**
   ```bash
   flyctl deploy --build-only --push
   ```
   
1. **Deploy the application:**
   ```bash
   flyctl deploy
   ```

2. **Open your deployed app:**
   ```bash
   flyctl open
   ```

## Admin

- **Run a one-off command:**
   ```bash
   fly ssh console -C "game-generator"
   ```

## Monitoring

- **View logs:**
  ```bash
  flyctl logs
  ```

- **Check app status:**
  ```bash
  flyctl status
  ```

- **Scale your app:**
  ```bash
  flyctl scale count 2
  ```

## Database Management

- **Connect to PostgreSQL:**
  ```bash
  flyctl postgres connect -a pathfinder-db
  ```

- **View database info:**
  ```bash
  flyctl postgres list
  ```

## Updating

To deploy updates:
```bash
flyctl deploy
```

## Configuration Notes

- Health checks are configured on `/health` endpoint
- Database migrations run automatically on startup

## Troubleshooting

- Check logs if deployment fails: `flyctl logs`
- Verify database connection: `flyctl postgres connect -a pathfinder-db`
- Restart the app: `flyctl apps restart pathfinder-game`
