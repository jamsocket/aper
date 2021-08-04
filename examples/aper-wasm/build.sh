#!/bin/sh

set -e

rm -f out.wasm out.wat

cargo build --release --target wasm32-unknown-unknown

wasm-opt -O2 target/wasm32-unknown-unknown/release/aper_wasm.wasm -o out.wasm

wasm2wat out.wasm -o out.wat

