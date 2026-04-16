# ============================================================
# Stage 1 — Build the Yew WASM app
# ============================================================
FROM rust:1.86-bookworm AS builder

# Limit parallelism and turn off LTO for the Yew app compile.
# The tools below are downloaded as prebuilt binaries — compiling
# trunk from source uses fat-LTO and kills the linker under low RAM.
ENV CARGO_BUILD_JOBS=1 \
    RUSTFLAGS="-C codegen-units=1 -C lto=off"

# Download prebuilt trunk v0.21.5 (avoids ~1.5 GB fat-LTO link step)
RUN curl -sSfL \
    "https://github.com/trunk-rs/trunk/releases/download/v0.21.5/trunk-x86_64-unknown-linux-gnu.tar.gz" \
    | tar -xz -C /usr/local/cargo/bin/ trunk

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app
# Copy the frontend package and the shared logic crate it depends on.
# frontend_logic lives at ../frontend_logic relative to the package root,
# which cargo resolves as /frontend_logic when WORKDIR is /app.
COPY frontend/ .
COPY frontend_logic/ /frontend_logic/
RUN trunk build --release

# ============================================================
# Stage 2 — Generate self-signed TLS certificate
# ============================================================
FROM debian:bookworm-slim AS certgen
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
RUN mkdir /certs && openssl req -x509 -newkey rsa:2048 -nodes \
    -keyout /certs/server.key -out /certs/server.crt \
    -days 3650 -subj "/CN=frontend/O=TourismPortal/C=US"

# ============================================================
# Stage 3 — Serve the pre-built dist with nginx over HTTPS
#
# nginx replaces trunk serve: it is a stable static-file server
# with no WASM rebuild, no HMR WebSocket, and no risk of OOM
# killing the container during the test run.
# ============================================================
FROM nginx:1.25-alpine AS server

# Create directory for TLS certificates
RUN mkdir -p /etc/nginx/certs

# Copy pre-built WASM app from the builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy TLS certificates from certgen stage
COPY --from=certgen /certs/server.crt /etc/nginx/certs/server.crt
COPY --from=certgen /certs/server.key /etc/nginx/certs/server.key

# Write nginx site configuration (replaces the default HTTP config).
#
# proxy_pass uses a variable ($backend_upstream) so that nginx defers
# DNS resolution to request time via Docker's embedded resolver
# (127.0.0.11).  A static proxy_pass hostname would be resolved once at
# startup — and also during "nginx -t" at build time — which fails because
# the "backend" service does not exist in the build context.
RUN printf 'server {\n\
    listen 8081 ssl;\n\
    server_name _;\n\
\n\
    ssl_certificate     /etc/nginx/certs/server.crt;\n\
    ssl_certificate_key /etc/nginx/certs/server.key;\n\
    ssl_protocols       TLSv1.2 TLSv1.3;\n\
    ssl_ciphers         HIGH:!aNULL:!MD5;\n\
\n\
    root  /usr/share/nginx/html;\n\
    index index.html;\n\
\n\
    # Docker'\''s embedded DNS — required when proxy_pass uses a variable.\n\
    resolver 127.0.0.11 valid=30s ipv6=off;\n\
    set $backend_upstream "https://backend:8080";\n\
\n\
    # Proxy /api/ requests to the backend (self-signed cert, ssl_verify off).\n\
    # Using a variable defers resolution to request time instead of startup.\n\
    location /api/ {\n\
        proxy_pass          $backend_upstream;\n\
        proxy_ssl_verify    off;\n\
        proxy_set_header    Host              $host;\n\
        proxy_set_header    X-Real-IP         $remote_addr;\n\
        proxy_set_header    X-Forwarded-For   $proxy_add_x_forwarded_for;\n\
        proxy_set_header    X-Forwarded-Proto $scheme;\n\
    }\n\
\n\
    # SPA fallback: non-file paths serve index.html for hash-based routing.\n\
    location / {\n\
        try_files $uri $uri/ /index.html;\n\
    }\n\
}\n' > /etc/nginx/conf.d/default.conf

# Validate nginx configuration at build time (certs must exist before this)
RUN nginx -t

EXPOSE 8081
CMD ["nginx", "-g", "daemon off;"]
