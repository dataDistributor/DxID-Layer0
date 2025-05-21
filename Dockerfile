# ---- Build Stage ----
FROM rust:nightly-bullseye AS builder

# Install build deps (libssl, clang, cmake, etc.)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/dxid-layer0

# Copy manifests and lockfile for caching
COPY Cargo.toml Cargo.lock ./

# Copy your workspaces and source
COPY src        ./src
COPY smart-contracts ./smart-contracts
COPY scripts    ./scripts
COPY ddxid_chain.json ./

# Build the CLI binary (adjust `--bin` if yours is named differently)
RUN cargo build --release --bin dxid-node

# ---- Runtime Stage ----
FROM debian:bullseye-slim

# Install runtime deps only
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m dxiduser

# Copy the built binary + config into the slim image
COPY --from=builder /usr/src/dxid-layer0/target/release/dxid-node /usr/local/bin/dxid-node
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json /etc/ddxid_chain.json

USER dxiduser

# Expose your p2p port (and add any HTTP port if you have one)
EXPOSE 30333

# Default to running your node with the supplied chain spec
ENTRYPOINT ["dxid-node", "--config", "/etc/ddxid_chain.json"]
