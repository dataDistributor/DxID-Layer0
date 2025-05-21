# ==== Build Stage ====
FROM rust:1.76-slim AS builder

# 1. Install system deps + nightly toolchain + rustfmt
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

# 2. Set working dir
WORKDIR /usr/src/dxid-layer0

# 3. Copy the *entire* project into the container
COPY . .

# 4. Cache dependencies
RUN cargo fetch

# 5. Build your CLI binary
RUN cargo build --release --bin dxid-node

# ==== Runtime Stage ====
FROM debian:bullseye-slim

# 6. Install runtime deps
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 7. Create non-root user
RUN useradd -m dxiduser

# 8. Copy the compiled binary + chain spec
COPY --from=builder /usr/src/dxid-layer0/target/release/dxid-node /usr/local/bin/dxid-node
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json /etc/ddxid_chain.json

# 9. Switch to non-root
USER dxiduser

# 10. Expose P2P port (and HTTP if you add one later)
EXPOSE 30333

# 11. Launch
ENTRYPOINT ["dxid-node", "--config", "/etc/ddxid_chain.json"]
