# ---- Build Stage ----
FROM rust:1.76-slim AS builder

# 1. Install build deps + nightly for edition2024 support
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/dxid-layer0

# 2. Copy your entire workspace
COPY . .

# 3. Build just the node binary
RUN cargo build --release --bin layer0-core

# ---- Runtime Stage ----
FROM debian:bookworm-slim

# 4. Runtime-only deps
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 5. Non‑root user
RUN useradd -m dxiduser

# 6. Copy the built binary + config
COPY --from=builder /usr/src/dxid-layer0/target/release/layer0-core /usr/local/bin/layer0-core
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json    /etc/ddxid_chain.json

USER dxiduser

# 7. Expose P2P port
EXPOSE 30333

# 8. Launch your node
ENTRYPOINT ["layer0-core", "--config", "/etc/ddxid_chain.json"]
