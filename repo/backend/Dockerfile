# ============================================================
# Stage 1 — Generate a build recipe with cargo-chef
# ============================================================
FROM rust:1.78-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# ============================================================
# Stage 2 — Plan (fingerprint dependencies)
# ============================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ============================================================
# Stage 3 — Build dependencies (cached layer) + final binary
# ============================================================
FROM chef AS builder

# Install system deps needed for static linking and openssl
RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    libssl-dev \
    libpq-dev \
    perl \
    make \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

# Cook dependencies from the recipe (layer-cached)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Copy source and build the real binary
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# ============================================================
# Stage 4 — Generate self-signed TLS certificates
# ============================================================
FROM debian:bookworm-slim AS certgen
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
RUN mkdir /certs && openssl req -x509 -newkey rsa:2048 -nodes \
    -keyout /certs/server.key -out /certs/server.crt \
    -days 3650 -subj "/CN=tourism-portal/O=TourismPortal/C=US"

# ============================================================
# Stage 5 — Minimal final image (scratch + ca-certs via alpine)
# ============================================================
FROM alpine:3.19 AS scratch-base
RUN apk add --no-cache ca-certificates coreutils && \
    mkdir -p /app/certs /app/uploads/imports

FROM scratch

COPY --from=scratch-base /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=scratch-base /usr/bin/df /usr/bin/df
COPY --from=scratch-base /lib/ld-musl-x86_64.so.1 /lib/ld-musl-x86_64.so.1
COPY --from=scratch-base /app /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/tourism-backend /app/tourism-backend
COPY --from=certgen /certs/server.crt /app/certs/server.crt
COPY --from=certgen /certs/server.key /app/certs/server.key
COPY config.toml /app/config.toml

WORKDIR /app
EXPOSE 8080

ENTRYPOINT ["/app/tourism-backend"]
