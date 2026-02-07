# Build stage
FROM docker.io/rustlang/rust:nightly-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates build-essential && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source code
COPY . .

# Build the application with caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/backend /backend

# Runtime stage
FROM docker.io/library/debian:trixie-slim

WORKDIR /app

# Install runtime dependencies (OpenSSL, etc.)
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /backend /app/backend

# Expose the port
EXPOSE 4000

# Set the entrypoint
CMD ["./backend"]