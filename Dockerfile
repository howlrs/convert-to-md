# syntax=docker/dockerfile:1

# ─── Stage 1: Build ───────────────────────────────────────────────────────────
FROM rust:1.85-slim AS builder

WORKDIR /app

# Cache dependencies separately
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/convert-to-md-rs target/release/deps/convert*

COPY src ./src
RUN cargo build --release

# ─── Stage 2: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim

# poppler-utils  → pdftotext (PDF conversion)
# libssl3        → TLS (future use)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        poppler-utils \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/convert-to-md-rs /usr/local/bin/convert-to-md-rs

# Default mount points
VOLUME ["/app/resources", "/app/data/output/markdown"]
WORKDIR /app

ENTRYPOINT ["convert-to-md-rs"]
CMD ["--input", "/app/resources", "--output", "/app/data/output/markdown"]
