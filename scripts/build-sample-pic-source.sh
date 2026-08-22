#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
target_dir="$root/pic-source-plugins/sample-hello-pic-source"
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
cargo build --manifest-path "$target_dir/Cargo.toml" --target wasm32-unknown-unknown --release
cp "$root/target/wasm32-unknown-unknown/release/sample_hello_pic_source.wasm" \
  "$target_dir/pic_source_plugin.wasm"
echo "Built $target_dir/pic_source_plugin.wasm"
