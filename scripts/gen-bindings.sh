#!/usr/bin/env bash
# Generate TypeScript bindings from the built WASM. Output is generated: never hand-edit.
set -euo pipefail
cd "$(dirname "$0")/.."
stellar contract build
stellar contract bindings typescript \
  --wasm target/wasm32v1-none/release/kinlock.wasm \
  --output-dir bindings/typescript \
  --overwrite
