# ============================================================
# Stage 1 — Build the Yew WASM app
# ============================================================
FROM rust:1.78-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked
RUN cargo install wasm-bindgen-cli --locked

WORKDIR /app
COPY . .
RUN trunk build --release index.html

# ============================================================
# Stage 2 — Serve with Trunk
# ============================================================
FROM rust:1.78-bookworm AS server

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked

WORKDIR /app
COPY --from=builder /app .

EXPOSE 8081

CMD ["trunk", "serve", "--release", "--address", "0.0.0.0", "--port", "8081", "--proxy-backend=http://backend:8080/api/", "index.html"]
