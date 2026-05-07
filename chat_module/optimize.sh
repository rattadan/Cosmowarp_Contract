#!/bin/bash

# Build optimized wasm with rust-optimizer 0.17.0
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.17.0

echo "✅ Optimized wasm created in artifacts/ directory"
ls -lh artifacts/
