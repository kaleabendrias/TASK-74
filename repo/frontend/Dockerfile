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

# Download prebuilt wasm-bindgen-cli v0.2.100
RUN curl -sSfL \
    "https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.100/wasm-bindgen-0.2.100-x86_64-unknown-linux-musl.tar.gz" \
    | tar -xz -C /tmp/ \
    && mv /tmp/wasm-bindgen-0.2.100-x86_64-unknown-linux-musl/wasm-bindgen \
          /usr/local/cargo/bin/ \
    && rm -rf /tmp/wasm-bindgen-0.2.100-x86_64-unknown-linux-musl

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY . .
RUN trunk build --release

# ============================================================
# Stage 2 — Serve with Trunk
# ============================================================
FROM rust:1.86-bookworm AS server

RUN rustup target add wasm32-unknown-unknown

# Reuse the prebuilt trunk binary from the builder stage — no recompile.
COPY --from=builder /usr/local/cargo/bin/trunk /usr/local/cargo/bin/trunk

WORKDIR /app
COPY --from=builder /app .

EXPOSE 8081

CMD ["trunk", "serve", "--release", "--address", "0.0.0.0", "--port", "8081", \
     "--proxy-backend=http://backend:8080/api/", "index.html"]
