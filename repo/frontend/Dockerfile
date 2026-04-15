# ============================================================
# Stage 1 — Build the Yew WASM app
# ============================================================
FROM rust:1.86-bookworm AS builder

# Limit parallelism so the OOM killer does not target the build.
# codegen-units=1 reduces peak memory per crate at the cost of
# slightly slower compile — acceptable in CI/Docker.
ENV CARGO_BUILD_JOBS=1 \
    RUSTFLAGS="-C codegen-units=1"

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk@0.21.5 --locked
RUN cargo install wasm-bindgen-cli@0.2.100 --locked

WORKDIR /app
COPY . .
RUN trunk build --release

# ============================================================
# Stage 2 — Serve with Trunk
# ============================================================
FROM rust:1.86-bookworm AS server

RUN rustup target add wasm32-unknown-unknown

# Reuse the trunk binary compiled in the builder stage instead
# of rebuilding it — avoids a second full compile under memory pressure.
COPY --from=builder /usr/local/cargo/bin/trunk /usr/local/cargo/bin/trunk

WORKDIR /app
COPY --from=builder /app .

EXPOSE 8081

CMD ["trunk", "serve", "--release", "--address", "0.0.0.0", "--port", "8081", "--proxy-backend=http://backend:8080/api/", "index.html"]
