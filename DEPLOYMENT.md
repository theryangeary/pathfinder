# Word Game Deployment Guide

This document provides instructions for running the word game application in local development and production environments.

## Architecture

The application consists of:
- **Frontend**: React/TypeScript application with Vite
- **Backend**: Rust HTTP API server with Postgres database
- **Game Generator and other cron-like bins**: Rust cron-like to generate games for missing days and store in database
- **Database**: PostgreSQL

## Local Development

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Postgres

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
# Install dependencies and build
cargo build

# Set up environment (if needed)
cp .env.example .env
# Edit .env with local settings

# Start backend server (migrations run automatically)
cargo run
```

The backend will start the HTTP API server on `http://localhost:3001`

## Production Deployment

### Environment Variables

Required environment variables:

```env
# Database
DATABASE_URL=postgresql://...

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
# Build optimized release
cargo build --release

# Run production server
./target/release/api-server

# Run game generator
./target/release/game-generator
```

### Docker Deployment (Optional)

Use docker compose to run a full suite of services:
```bash
docker compose up -d

# to rebuild docker image
docker compose up --build -d

# to stop
docker compose down
```

## Monitoring and Logging

### Logs

Backend logs are written to stdout with structured logging via `tracing`.

View logs:
```bash
# Development
cargo run 2>&1 | grep "pathfinder"

# Production on fly.io
fly logs -a pathfinder-game
```

### Health Checks

The backend provides monitoring through:
- Database connectivity check in startup logs
- HTTP API endpoint availability

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

## Troubleshooting

### Common Issues

1. **Database connection errors**:
   - Check DATABASE_URL format
   - Verify database file permissions
   - Ensure SQLite file exists

2. **HTTP API connection issues**:
   - Verify port 3001 is open
   - Check CORS configuration
   - Validate frontend/backend versions match

3. **Game generation failures**:
   - Check word list file exists
   - Verify random seed generation
   - Monitor threshold score settings

### Debug Commands

```bash
# Check database contents
docker exec -it pathfinder-postgres psql -U pathfinder
> SELECT * FROM games ORDER BY created_at DESC LIMIT 5;

# Test HTTP API endpoint
curl http://localhost:3001/api/game

# Monitor logs
tail -f src/api/backend.log
```

## Backup and Recovery

### Database Backup
```bash
# PostgreSQL backup
pg_dump wordgame > backup_$(date +%Y%m%d).sql
```

### Recovery
```bash
# PostgreSQL restore
psql wordgame < backup_20231215.sql
```

## Contact

For deployment issues or questions, refer to:
- Backend logs for detailed error information
- SPEC.md for system requirements
- CLAUDE.md for development guidelines
