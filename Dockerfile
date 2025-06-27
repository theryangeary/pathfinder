# Build stage for frontend
FROM node:18-alpine AS frontend-builder

WORKDIR /app
COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

# Build stage for backend
FROM rust:1.87-alpine AS backend-builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /app
COPY src/api/Cargo.toml Cargo.lock ./
COPY src/api/src ./src
COPY src/api/migrations ./migrations
COPY wordlist ./wordlist
RUN cargo build --release --bin api-server --bin game-generator

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache ca-certificates dcron curl

WORKDIR /app

# Copy both backend binaries
COPY --from=backend-builder /app/target/release/api-server ./api-server
COPY --from=backend-builder /app/target/release/game-generator ./game-generator

# Copy frontend static files
COPY --from=frontend-builder /app/src/web/dist ./static

# Copy wordlist and migrations
COPY --from=backend-builder /app/wordlist ./wordlist
COPY --from=backend-builder /app/migrations ./migrations

# Set environment variables for container deployment
ENV HTTP_PORT=8080
ENV SERVER_HOST=0.0.0.0
ENV PATH="${PATH}:/app"

EXPOSE 8080

CMD ["api-server"]
