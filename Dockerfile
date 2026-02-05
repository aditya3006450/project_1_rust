# =========================
# Stage 1: Build
# =========================
FROM rustlang/rust:nightly-slim AS builder
WORKDIR /app

# Install system dependencies (needed for libssl/pkg-config)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy everything (including the .sqlx folder we made locally)
COPY . .

# CRITICAL: Tell SQLx to use the .sqlx/ cache instead of connecting to a DB
ENV SQLX_OFFLINE=true

# Build release binary (This will be much faster now)
RUN cargo build --release

# =========================
# Stage 2: Runtime
# =========================
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m appuser
USER appuser

COPY --from=builder /app/target/release/project1_rust ./project1_rust
COPY --from=builder /app/templates ./templates

ENV PORT=3000
EXPOSE $PORT

CMD ["./project1_rust"]
