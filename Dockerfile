# ---- Build Stage ----
FROM rust:nightly-slim AS builder

# 1. Install system dependencies for building Rust + libp2p/zk‑STARKs
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

# 2. Create and set working directory
WORKDIR /usr/src/dxid-layer0

# 3. Copy manifest files and generate a lockfile if needed for caching
COPY Cargo.toml Cargo.lock ./
# (If you don’t want to commit Cargo.lock, replace the above two lines with:)
# COPY Cargo.toml ./
# RUN cargo generate-lockfile

# 4. Copy the rest of your workspace
COPY src ./src
COPY smart-contracts ./smart-contracts
COPY scripts ./scripts
COPY ddxid_chain.json ./

# 5. Build the release binary (adjust --bin to your CLI crate’s name)
RUN cargo build --release --bin dxid-node

# ---- Runtime Stage ----
FROM debian:bullseye-slim

# 6. Install only runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 7. Create a non‑root user
RUN useradd -m dxiduser

# 8. Copy the built binary and config
COPY --from=builder /usr/src/dxid-layer0/target/release/dxid-node /usr/local/bin/dxid-node
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json /etc/ddxid_chain.json

# 9. Switch to non‑root
USER dxiduser

# 10. Expose your P2P port (and any HTTP ports if used)
EXPOSE 30333

# 11. Default entrypoint
ENTRYPOINT ["dxid-node", "--config", "/etc/ddxid_chain.json"]
