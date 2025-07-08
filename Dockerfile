# Build stage for frontend
FROM node:18-alpine AS frontend-builder

WORKDIR /app
COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

# Build stage for backend
FROM rust:1.87-alpine AS backend-builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev build-base g++ cmake freetype-dev freetype-static fontconfig-dev fontconfig-static

WORKDIR /app
COPY src/api/Cargo.toml Cargo.lock ./
COPY src/api/src ./src
COPY src/api/migrations ./migrations
COPY wordlist ./wordlist
RUN cargo build --release --bin api-server --bin game-generator --bin stat-poster

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache ca-certificates dcron curl

# Latest releases available at https://github.com/aptible/supercronic/releases
ENV SUPERCRONIC_URL=https://github.com/aptible/supercronic/releases/download/v0.2.29/supercronic-linux-amd64 \
    SUPERCRONIC=supercronic-linux-amd64 \
    SUPERCRONIC_SHA1SUM=cd48d45c4b10f3f0bfdd3a57d054cd05ac96812b

RUN curl -fsSLO "$SUPERCRONIC_URL" \
 && echo "${SUPERCRONIC_SHA1SUM}  ${SUPERCRONIC}" | sha1sum -c - \
 && chmod +x "$SUPERCRONIC" \
 && mv "$SUPERCRONIC" "/usr/local/bin/${SUPERCRONIC}" \
 && ln -s "/usr/local/bin/${SUPERCRONIC}" /usr/local/bin/supercronic

WORKDIR /app

# Copy both backend binaries
COPY --from=backend-builder /app/target/release/api-server ./api-server
COPY --from=backend-builder /app/target/release/game-generator ./game-generator
COPY --from=backend-builder /app/target/release/stat-poster ./stat-poster

# Copy frontend static files
COPY --from=frontend-builder /app/src/web/dist ./static

# Copy wordlist and migrations
COPY --from=backend-builder /app/wordlist ./wordlist
COPY --from=backend-builder /app/migrations ./migrations

COPY crontab crontab

# Set environment variables for container deployment
ENV HTTP_PORT=8080
ENV SERVER_HOST=0.0.0.0
ENV PATH="${PATH}:/app"

EXPOSE 8080

CMD ["api-server"]
