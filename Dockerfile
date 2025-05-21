# ---- Build Stage ----
FROM rust:1.76-slim AS builder

# 1) Install build tools + Rust nightly for edition2024 if needed
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev build-essential clang cmake protobuf-compiler git curl && \
    rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rustfmt && \
    rm -rf /var/lib/apt/lists/*

# 2) Copy everything so Cargo can resolve workspace deps
WORKDIR /usr/src/dxid-layer0
COPY . .

# 3) Build the *exact* crate you run locally:
#    --manifest-path points at src/layer0-core/Cargo.toml
RUN cargo build --release \
    --manifest-path src/layer0-core/Cargo.toml \
    --bin layer0-core

# ---- Runtime Stage ----
FROM debian:bookworm-slim

# 4) Runtime libs only (glibc ≥2.34)
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 5) Non‑root user
RUN useradd -m dxiduser

# 6) Copy in the HTTP server binary + your chain spec
COPY --from=builder /usr/src/dxid-layer0/target/release/layer0-core /usr/local/bin/layer0-core
COPY --from=builder /usr/src/dxid-layer0/ddxid_chain.json    /etc/ddxid_chain.json

USER dxiduser

# 7) Expose the HTTP port your API listens on
EXPOSE 3030

# 8) Run the same command you use locally
ENTRYPOINT ["layer0-core", "--config", "/etc/ddxid_chain.json"]
