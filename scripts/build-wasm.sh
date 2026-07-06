#!/usr/bin/env bash
set -euo pipefail

if ! rustup target list --installed | grep -q '^wasm32-unknown-unknown$'; then
  echo "Missing wasm32-unknown-unknown target. Install it with:"
  echo "  rustup target add wasm32-unknown-unknown"
  exit 1
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "Missing wasm-bindgen CLI. Install it with:"
  echo "  cargo install wasm-bindgen-cli"
  exit 1
fi

RUSTC="$(rustup which --toolchain stable rustc)" \
  rustup run stable cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen \
  --target web \
  --out-dir web/pkg \
  target/wasm32-unknown-unknown/release/thronglets.wasm

echo "Built web/pkg. Serve ./web with any static file server."
