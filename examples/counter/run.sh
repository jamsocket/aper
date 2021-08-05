#!/bin/sh

set -e

OUTPUT_WASM="client/target/wasm32-wasi/release/client.wasm"

# Build

cargo build --manifest-path=client/Cargo.toml --release --target wasm32-wasi
wasm-opt -O2 "${OUTPUT_WASM}" -o _tmp.wasm
wasm2wat _tmp.wasm -o _tmp.wasm
mv _tmp.wasm "${OUTPUT_WASM}"

# Run

cargo run --release
