# ---- Build Stage ----
FROM rust:1.76-slim AS builder

# 1. Install system deps + nightly toolchain + rustfmt
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/dxid-layer0

# 2. Copy root manifests
COPY Cargo.toml Cargo.lock ./

# 3. Copy *only* the workspace member manifests (so cargo can resolve deps)
#    (adjust these paths if you add more crates later)
COPY src/layer0-core/Cargo.toml      src/layer0-core/Cargo.toml
COPY src/zk-verification/Cargo.toml  src/zk-verification/Cargo.toml
COPY smart-contracts/identity/Cargo.toml smart-contracts/identity/Cargo.toml

# 4. Fetch all dependencies (this will cache on manifest changes)
RUN cargo fetch

# 5. Now copy the full source tree
COPY src           ./src
COPY smart-contracts ./smart-contracts
COPY scripts       ./scripts
COPY ddxid_chain.json ./

# 6. Build your CLI binary
RUN cargo build --release --bin dxid-node

# ---- Runtime Stage ----
FROM debian:bullseye-slim

# 7. Runtime deps only
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 8. Create non‐root user
RUN useradd -m dxiduser

# 9. Copy the compiled binary & chain spec
COPY --from=builder /usr/src/dxid-layer0/target/release/dxid-node /usr/local/bin/dxid-node
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json /etc/ddxid_chain.json

# 10. Switch to non‐root
USER dxiduser

# 11. Expose P2P port (and HTTP if you add one)
EXPOSE 30333

# 12. Launch
ENTRYPOINT ["dxid-node", "--config", "/etc/ddxid_chain.json"]
