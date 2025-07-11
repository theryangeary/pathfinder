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

COPY src/api/Cargo.toml Cargo.lock ./
COPY src/api/src ./src
COPY src/api/migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates curl gnupg unzip \
 && curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor -o /usr/share/keyrings/postgresql-keyring.gpg \
 && echo "deb [signed-by=/usr/share/keyrings/postgresql-keyring.gpg] https://apt.postgresql.org/pub/repos/apt bookworm-pgdg main" > /etc/apt/sources.list.d/postgresql.list \
 && apt-get update && apt-get install -y postgresql-client-17

# Latest releases available at https://github.com/aptible/supercronic/releases
ENV SUPERCRONIC_URL=https://github.com/aptible/supercronic/releases/download/v0.2.29/supercronic-linux-amd64 \
    SUPERCRONIC=supercronic-linux-amd64 \
    SUPERCRONIC_SHA1SUM=cd48d45c4b10f3f0bfdd3a57d054cd05ac96812b

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
COPY --from=builder /app/migrations ./migrations
COPY crontab crontab


# verify crontab
RUN supercronic -test crontab

# Set environment variables for container deployment
ENV HTTP_PORT=8080
ENV SERVER_HOST=0.0.0.0
ENV PATH="${PATH}:/app"

EXPOSE 8080

CMD ["api-server"]
