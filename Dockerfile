# ---- Build Stage ----
FROM rust:1.76-slim AS builder

# 1. Install build tools & Rust fmt
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

# 2. Set working directory
WORKDIR /usr/src/dxid-layer0

# 3. Copy your entire project (workspace + manifests + chain spec)
COPY . .

# 4. Build the workspace under nightly
RUN cargo build --release --bin dxid-node

# ---- Runtime Stage ----
FROM debian:bullseye-slim

# 5. Runtime dependencies only
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 6. Non-root user
RUN useradd -m dxiduser

# 7. Copy the compiled binary and chain spec
COPY --from=builder /usr/src/dxid-layer0/target/release/dxid-node /usr/local/bin/dxid-node
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json  /etc/ddxid_chain.json

# 8. Drop privileges
USER dxiduser

# 9. Expose whatever ports you need (P2P port, HTTP port if any)
EXPOSE 30333

# 10. Default command
ENTRYPOINT ["dxid-node", "--config", "/etc/ddxid_chain.json"]
