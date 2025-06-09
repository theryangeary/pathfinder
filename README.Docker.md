# Docker Setup for Pathfinder

This document explains how to run the Pathfinder word game using Docker and Docker Compose.

## Prerequisites

- Docker and Docker Compose installed on your system
- At least 2GB of available disk space

## Quick Start

1. **Build and start all services:**
   ```bash
   docker-compose up --build
   ```

2. **Access the application:**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:3001
   - PostgreSQL: localhost:5432

3. **Stop all services:**
   ```bash
   docker-compose down
   ```

## Services

### PostgreSQL Database
- **Image:** postgres:15-alpine
- **Port:** 5432
- **Database:** pathfinder
- **User:** pathfinder
- **Password:** pathfinder_pass
- **Volume:** postgres_data (persistent storage)

### Backend (Rust API)
- **Build:** src/api/Dockerfile
- **Port:** 3001
- **Dependencies:** PostgreSQL
- **Environment:** Production-ready configuration

### Frontend (React App)
- **Build:** src/web/Dockerfile
- **Port:** 3000 (nginx)
- **Dependencies:** Backend
- **Features:** Nginx reverse proxy to backend

## Development vs Production

### Development (recommended for coding)
```bash
# Terminal 1: Start PostgreSQL only
docker-compose up postgres

# Terminal 2: Run backend locally
cd src/api
cargo run

# Terminal 3: Run frontend locally
cd src/web
npm run dev
```

### Production (containerized)
```bash
# Start all services
docker-compose up --build
```

## Environment Variables

The docker-compose.yml includes production-ready environment variables:

- **Database:** Full PostgreSQL connection string
- **CORS:** Configured for containerized setup
- **Security:** Rate limiting and security headers enabled
- **Logging:** Info level logging

## Volumes and Data Persistence

- **postgres_data:** PostgreSQL data is persisted between container restarts
- **Wordlist:** Backend wordlist is mounted read-only from host

## Troubleshooting

### Container build issues
```bash
# Clean rebuild
docker-compose down
docker-compose build --no-cache
docker-compose up
```

### Database connection issues
```bash
# Check if PostgreSQL is healthy
docker-compose ps
docker-compose logs postgres
```

### Port conflicts
If ports 3000, 3001, or 5432 are in use, modify the port mappings in docker-compose.yml:
```yaml
ports:
  - "8080:80"  # Change frontend port
  - "8001:3001"  # Change backend port
  - "5433:5432"  # Change PostgreSQL port
```

## Production Deployment

For production deployment:

1. **Update environment variables** in docker-compose.yml
2. **Set up SSL/TLS** termination (nginx or load balancer)
3. **Configure proper CORS origins**
4. **Set up backup strategy** for PostgreSQL data
5. **Monitor logs and metrics**

Example production environment updates:
```yaml
environment:
  ALLOWED_ORIGINS: https://yourdomain.com
  STRICT_REFERER: "true"
  RUST_LOG: warn
```