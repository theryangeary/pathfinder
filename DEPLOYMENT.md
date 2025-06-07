# Word Game Deployment Guide

This document provides instructions for running the word game application in local development and production environments.

## Architecture

The application consists of:
- **Frontend**: React/TypeScript application with Vite
- **Backend**: Rust gRPC server with SQLite database
- **Database**: SQLite (development) → PostgreSQL (production)

## Local Development

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- SQLite3

### Frontend Development

```bash
# From project root, install dependencies
npm install

# Start development server
npm run dev
```

The frontend will be available at `http://localhost:5173` with hot reloading enabled.

### Backend Development

```bash
# Navigate to backend directory from project root
cd src/api

# Install dependencies and build
cargo build

# Set up environment (if needed)
cp .env.example .env
# Edit .env with local settings

# Start backend server (migrations run automatically)
cargo run
```

The backend will start:
- HTTP API server on `http://localhost:3001`
- gRPC server on `http://localhost:50051`

### Database Setup

The database is automatically initialized on first run. For manual setup:
```bash
# Navigate to backend directory from project root
cd src/api

# Create SQLite database manually (optional)
sqlite3 game.db < migrations/001_initial.sql

# Verify schema
sqlite3 game.db ".schema"
```

## Production Deployment

### Environment Variables

Required environment variables:

```env
# Database
DATABASE_URL=sqlite://game.db  # or postgresql://...

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=50051

# Logging
RUST_LOG=info
```

### Frontend Production Build

```bash
# Build for production
npm run build

# Serve static files (example with nginx)
# Point nginx document root to ./dist
```

### Backend Production Deployment

```bash
# Navigate to backend directory from project root
cd src/api

# Build optimized release
cargo build --release

# Run production server
./target/release/word-game-backend
```

### Database Migration (SQLite → PostgreSQL)

When scaling to PostgreSQL:

1. **Setup PostgreSQL**:
```sql
CREATE DATABASE wordgame;
CREATE USER wordgame_user WITH PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE wordgame TO wordgame_user;
```

2. **Update Environment**:
```env
DATABASE_URL=postgresql://wordgame_user:secure_password@localhost/wordgame
```

3. **Run Migrations**:
```bash
# Update connection in src/api/src/db/mod.rs to use PostgreSQL
# Navigate to backend directory from project root and run migrations
cd src/api
cargo run
```

### Docker Deployment (Optional)

Create `Dockerfile` for backend:
```dockerfile
FROM rust:1.70 AS builder
WORKDIR /app
COPY src/api .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/word-game-backend /usr/local/bin/
EXPOSE 3001 50051
CMD ["word-game-backend"]
```

Create `docker-compose.yml`:
```yaml
version: '3.8'
services:
  backend:
    build: 
      context: .
      dockerfile: src/api/Dockerfile
    ports:
      - "3001:3001"
      - "50051:50051"
    environment:
      - DATABASE_URL=sqlite://game.db
      - RUST_LOG=info
    volumes:
      - ./data:/app/data

  frontend:
    build: .
    ports:
      - "80:80"
    depends_on:
      - backend
```

## Monitoring and Logging

### Logs

Backend logs are written to stdout with structured logging via `tracing`.

View logs:
```bash
# Development
cargo run 2>&1 | grep "word-game-backend"

# Production with systemd
journalctl -u word-game-backend -f
```

### Health Checks

The backend exposes these endpoints for monitoring:
- gRPC health check via `grpc_health_probe`
- Database connectivity check in startup logs

### Performance Monitoring

Key metrics to monitor:
- Game generation success rate
- Database query performance  
- Daily active users
- Game completion rates

## Security Considerations

1. **Database**: Use connection pooling and prepared statements (already implemented)
2. **Authentication**: Cookie-based user sessions (low security but appropriate for game)
3. **CORS**: Configured to allow frontend origin
4. **Rate Limiting**: Consider adding for production
5. **HTTPS**: Terminate TLS at load balancer/proxy level

## Scaling Considerations

### Database Scaling
- **0-10K users**: SQLite sufficient
- **10K+ users**: Migrate to PostgreSQL
- **100K+ users**: Consider read replicas
- **1M+ users**: Implement database sharding

### Application Scaling
- **Horizontal scaling**: Run multiple backend instances behind load balancer
- **Caching**: Add Redis for game state caching
- **CDN**: Use CDN for frontend static assets

### Game Generation
- Current: Single-threaded generation at startup + cron
- Scale: Move to distributed queue (Redis + workers)
- Advanced: Pre-generate games in batches

## Troubleshooting

### Common Issues

1. **Database connection errors**:
   - Check DATABASE_URL format
   - Verify database file permissions
   - Ensure SQLite file exists

2. **gRPC connection issues**:
   - Verify port 50051 is open
   - Check CORS configuration
   - Validate frontend/backend versions match

3. **Game generation failures**:
   - Check word list file exists
   - Verify random seed generation
   - Monitor threshold score settings

### Debug Commands

```bash
# Check database contents
cd src/api
sqlite3 game.db "SELECT * FROM games ORDER BY created_at DESC LIMIT 5;"

# Test HTTP API endpoint
curl http://localhost:3001/api/game

# Test gRPC endpoint
grpcurl -plaintext localhost:50051 wordgame.WordGameService/GetDailyGame

# Monitor logs
tail -f src/api/backend.log
```

## Backup and Recovery

### Database Backup
```bash
# SQLite backup
sqlite3 game.db ".backup backup_$(date +%Y%m%d).db"

# PostgreSQL backup
pg_dump wordgame > backup_$(date +%Y%m%d).sql
```

### Recovery
```bash
# SQLite restore
cp backup_20231215.db game.db

# PostgreSQL restore
psql wordgame < backup_20231215.sql
```

## Contact

For deployment issues or questions, refer to:
- Backend logs for detailed error information
- SPEC.md for system requirements
- CLAUDE.md for development guidelines