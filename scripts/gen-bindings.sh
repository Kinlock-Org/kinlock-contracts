#!/usr/bin/env bash
# Generate the TypeScript bindings (@kinlock/contract) from the built WASM.
# The output in bindings/typescript is generated: never hand-edit it; change the contract and
# re-run this script. CI fails if the committed bindings differ from a fresh generation.
#   scripts/gen-bindings.sh [version]   # version defaults to the one already committed
set -euo pipefail
cd "$(dirname "$0")/.."

OUT=bindings/typescript
VERSION="${1:-$(jq -r '.version // "0.0.0"' "$OUT/package.json" 2>/dev/null || echo 0.0.0)}"

stellar contract build
stellar contract bindings typescript \
  --wasm target/wasm32v1-none/release/kinlock.wasm \
  --output-dir "$OUT" \
  --overwrite

# Package metadata (ADR-0023): public npm package under @kinlock, built output only.
# stellar-sdk is pinned to the version kinlock-sdk uses, so apps load a single copy.
tmp="$(mktemp)"
jq --arg version "$VERSION" '
  .name = "@kinlock/contract"
  | .version = $version
  | .description = "Generated TypeScript bindings for the Kinlock Soroban contract"
  | .license = "Apache-2.0"
  | .repository = {type: "git", url: "https://github.com/Kinlock-Org/kinlock-contracts", directory: "bindings/typescript"}
  | .files = ["dist"]
  | .publishConfig = {access: "public"}
  | .dependencies["@stellar/stellar-sdk"] = "17.2.1"
' "$OUT/package.json" > "$tmp"
mv "$tmp" "$OUT/package.json"
