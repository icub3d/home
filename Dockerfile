# --- Frontend Build Stage ---
FROM docker.io/library/node:22-alpine as frontend-builder
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY frontend/ .
# Set API URL to relative path for unified serving
ENV VITE_API_URL=/api
RUN npm run build

# --- Backend Build Stage ---
FROM docker.io/rustlang/rust:nightly-slim as backend-builder
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates build-essential && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY backend/ .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/backend /backend

# --- Runtime Stage ---
FROM docker.io/library/debian:trixie-slim
WORKDIR /app
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy backend binary
COPY --from=backend-builder /backend /app/backend

# Copy frontend assets to public directory
COPY --from=frontend-builder /app/dist /app/public

# Environment defaults for container
ENV SERVE_FRONTEND=true
ENV FRONTEND_DIST_DIR=/app/public
ENV DATABASE_URL=sqlite:/app/data/home.db
ENV PHOTOS_DIR=/app/data/photos

EXPOSE 4000
CMD ["./backend"]
