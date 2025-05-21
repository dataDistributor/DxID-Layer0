# ---- Build Stage ----
FROM rust:1.76-slim AS builder

# 1. Install build tools + switch to nightly for edition2024
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

# 2. Copy your entire project in so Cargo sees the workspace
WORKDIR /usr/src/dxid-layer0
COPY . .

# 3. Build just the layer0-core binary
RUN cargo build --release --bin layer0-core

# ---- Runtime Stage ----
FROM debian:bullseye-slim

# 4. Install only runtime deps
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 5. Non-root user for safety
RUN useradd -m dxiduser

# 6. Copy the compiled node binary and your chain spec
COPY --from=builder /usr/src/dxid-layer0/target/release/layer0-core /usr/local/bin/layer0-core
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json    /etc/ddxid_chain.json

USER dxiduser

# 7. Expose your P2P port (and any HTTP port if you add one)
EXPOSE 30333

# 8. Run your node
ENTRYPOINT ["layer0-core", "--config", "/etc/ddxid_chain.json"]
