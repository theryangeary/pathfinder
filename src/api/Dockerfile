# Backend Dockerfile for Rust API
FROM rust:1.87 as builder

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src
COPY migrations ./migrations
COPY wordlist ./wordlist

# Build the application
RUN touch src/main.rs && cargo build --release

# Production stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/word-game-backend /app/word-game-backend

# Copy wordlist and migrations
COPY --from=builder /app/wordlist /app/wordlist
COPY --from=builder /app/migrations /app/migrations

# Expose port 3001
EXPOSE 3001

# Run the application
CMD ["./word-game-backend"]