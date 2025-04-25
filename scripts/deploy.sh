#!/bin/bash
set -e

echo "Building Rust projects..."
cd src/layer0-core
cargo build --release
cd - || exit

cd src/zk-verification
cargo build --release
cargo test --release
cd - || exit

cd smart-contracts/identity
cargo build --release
cargo test --release
cd - || exit

echo "Building C++ dApp..."
cd dapp-cpp/build || { echo "C++ build directory not found. Please run 'mkdir build' inside dapp-cpp."; exit 1; }
cmake ..
make
cd - || exit

echo "All components built successfully."
