# build frontend
FROM node:24 as frontend
WORKDIR /app
COPY . .
RUN npm install
RUN npm run build

# use chef for faster rust builds/better caching
FROM lukemathwalker/cargo-chef:latest-rust-1.87 AS chef
WORKDIR /app

# generate chef plan
FROM chef AS planner

COPY src/api/Cargo.toml Cargo.lock ./
COPY src/api/src ./src
COPY src/api/migrations ./migrations

RUN cargo chef prepare --recipe-path recipe.json

# build rust bins
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

RUN mkdir -p /app/src/api
COPY src/api/Cargo.toml /app/src/api
COPY Cargo.lock Cargo.toml .
COPY src/api/src /app/src/api/src
COPY src/api/migrations /app/src/api/migrations

COPY --from=frontend /app/src/web/dist /app/src/web/dist

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates curl gnupg unzip sqlite3

# Latest releases available at https://github.com/aptible/supercronic/releases
ENV SUPERCRONIC_URL=https://github.com/aptible/supercronic/releases/download/v0.2.34/supercronic-linux-arm64 \
    SUPERCRONIC_SHA1SUM=4ab6343b52bf9da592e8b4bb7ae6eb5a8e21b71e \
    SUPERCRONIC=supercronic-linux-arm64

RUN curl -fsSLO "$SUPERCRONIC_URL" \
 && echo "${SUPERCRONIC_SHA1SUM}  ${SUPERCRONIC}" | sha1sum -c - \
 && chmod +x "$SUPERCRONIC" \
 && mv "$SUPERCRONIC" "/usr/local/bin/${SUPERCRONIC}" \
 && ln -s "/usr/local/bin/${SUPERCRONIC}" /usr/local/bin/supercronic

# Install rclone
RUN curl -fsSL https://rclone.org/install.sh | bash

WORKDIR /app

# Copy all backend binaries
COPY --from=builder /app/target/release/api-server ./api-server
COPY --from=builder /app/target/release/stat-poster ./stat-poster
COPY --from=builder /app/target/release/game-ender ./game-ender
COPY --from=builder /app/target/release/game-generator ./game-generator
COPY --from=builder /app/target/release/run-migrations ./run-migrations

# Copy control scripts
COPY scripts/setup_rclone.sh setup_rclone.sh
COPY scripts/store_backup.sh store_backup.sh
COPY scripts/restore_backup.sh restore_backup.sh
COPY scripts/cron_entrypoint.sh cron_entrypoint.sh

# Copy static resources
COPY wordlist wordlist
# COPY --from=builder /app/migrations ./migrations
COPY crontab crontab


# verify crontab
RUN supercronic -test crontab

# Set environment variables for container deployment
ENV HTTP_PORT=8080
ENV SERVER_HOST=0.0.0.0
ENV PATH="${PATH}:/app"

EXPOSE 8080

CMD ["api-server"]
