#!/bin/bash

# Build optimized wasm
cargo build --release --target wasm32-unknown-unknown

# Optimize wasm
wasm-opt -Os -o artifacts/chat_module.wasm target/wasm32-unknown-unknown/release/chat_module.wasm

echo "Build complete: artifacts/chat_module.wasm"
