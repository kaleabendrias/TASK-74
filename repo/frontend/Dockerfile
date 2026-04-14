# ============================================================
# Stage 1 — Build the Yew WASM app
# ============================================================
FROM rust:1.86-bookworm AS builder

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
RUN cargo install trunk@0.21.5 --locked

WORKDIR /app
COPY --from=builder /app .

EXPOSE 8081

CMD ["trunk", "serve", "--release", "--address", "0.0.0.0", "--port", "8081", "--proxy-backend=http://backend:8080/api/", "index.html"]
