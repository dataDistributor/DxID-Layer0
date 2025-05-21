# ---- Build Stage ----
FROM rust:1.76-slim AS builder

# 1. Install system deps + rustfmt + nightly toolchain
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/dxid-layer0

# 2. Copy manifests & lockfile
COPY Cargo.toml Cargo.lock ./

# 3. Cache dependencies
RUN cargo fetch

# 4. Copy your source
COPY src             ./src
COPY smart-contracts ./smart-contracts
COPY scripts         ./scripts
COPY ddxid_chain.json ./

# 5. Build under nightly
RUN cargo build --release --bin dxid-node

# ---- Runtime Stage ----
FROM debian:bullseye-slim

# 6. Runtime deps only
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -m dxiduser

# 7. Copy the built binary + config
COPY --from=builder /usr/src/dxid-layer0/target/release/dxid-node /usr/local/bin/dxid-node
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json /etc/ddxid_chain.json

USER dxiduser

EXPOSE 30333

ENTRYPOINT ["dxid-node", "--config", "/etc/ddxid_chain.json"]
