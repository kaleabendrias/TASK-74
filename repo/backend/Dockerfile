# ============================================================
# Stage 1 — Build the Rust binary
# ============================================================
FROM rust:1.86-bookworm AS builder

# Limit parallelism to prevent OOM during compilation.
# codegen-units=1 serialises LLVM codegen; combined with JOBS=1
# the peak RSS stays well under 2 GB even for large crates.
ENV CARGO_BUILD_JOBS=1 \
    RUSTFLAGS="-C codegen-units=1"

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# ============================================================
# Stage 2 — Generate self-signed TLS certificates
# ============================================================
FROM debian:bookworm-slim AS certgen
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
RUN mkdir /certs && openssl req -x509 -newkey rsa:2048 -nodes \
    -keyout /certs/server.key -out /certs/server.crt \
    -days 3650 -subj "/CN=tourism-portal/O=TourismPortal/C=US"

# ============================================================
# Stage 3 — Minimal runtime image
# ============================================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/certs /app/uploads/imports

COPY --from=builder /app/target/release/tourism-backend /app/tourism-backend
COPY --from=certgen /certs/server.crt /app/certs/server.crt
COPY --from=certgen /certs/server.key /app/certs/server.key
COPY config.toml /app/config.toml

WORKDIR /app
EXPOSE 8080

ENTRYPOINT ["/app/tourism-backend"]
