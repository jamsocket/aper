#!/bin/sh

set -e

# Build server

cargo build -p drop-four-service --target=wasm32-wasi

# Build client

wasm-pack build

jamsocket serve target/wasm32-wasi/debug/drop_four_service.wasm
