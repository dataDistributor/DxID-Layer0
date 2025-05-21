# ---- Build Stage ----
FROM rust:nightly AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/dxid-layer0

# Copy manifests & lockfile
COPY Cargo.toml Cargo.lock ./
# If you ever choose not to commit Cargo.lock, do:
# COPY Cargo.toml ./
# RUN cargo generate-lockfile

# Copy workspace sources
COPY src ./src
COPY smart-contracts ./smart-contracts
COPY scripts ./scripts
COPY ddxid_chain.json ./

# Build your CLI binary
RUN cargo build --release --bin dxid-node

# ---- Runtime Stage ----
FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -m dxiduser

COPY --from=builder /usr/src/dxid-layer0/target/release/dxid-node /usr/local/bin/dxid-node
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json /etc/ddxid_chain.json

USER dxiduser

EXPOSE 30333

ENTRYPOINT ["dxid-node", "--config", "/etc/ddxid_chain.json"]
