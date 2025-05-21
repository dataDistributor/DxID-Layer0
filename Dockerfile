# ---- Build Stage ----
FROM rust:1.76-slim AS builder

# 1. Install system deps for building Rust + libp2p/zk‑STARKs
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

# 2. Create and set workdir
WORKDIR /usr/src/dxid-layer0

# 3. Copy manifest files and fetch dependencies (best for Docker layer‑caching)
COPY Cargo.toml Cargo.lock ./
# If you’re using a workspace, also copy workspace members definitions
COPY src ./src
COPY smart-contracts ./smart-contracts
COPY scripts ./scripts
COPY ddxid_chain.json ./

# 4. Build the release binary (adjust --bin to match your CLI crate)
RUN cargo build --release --bin dxid-node

# ---- Runtime Stage ----
FROM debian:bullseye-slim

# 5. Install only runtime deps
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 6. Create non-root user for safety
RUN useradd -m dxiduser

# 7. Copy built binary and any config files
COPY --from=builder /usr/src/dxid-layer0/target/release/dxid-node /usr/local/bin/dxid-node
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json /etc/ddxid_chain.json

# 8. Switch to non‑root
USER dxiduser

# 9. Expose P2P and/or API ports (if your node also serves HTTP)
EXPOSE 30333

# 10. Default command (point to your config if needed)
ENTRYPOINT ["dxid-node", "--config", "/etc/ddxid_chain.json"]
