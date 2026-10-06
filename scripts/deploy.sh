#!/usr/bin/env bash
# [sec] Build, upload, and deploy the kinlock contract, then record the deployment.
#
#   STELLAR_ACCOUNT=<keystore identity name> KINLOCK_ADMIN=<G… or C… address> \
#     [STELLAR_NETWORK=testnet|local|mainnet] scripts/deploy.sh [--dry-run]
#
# - Defaults to testnet. `local` is the quickstart network from scripts/localnet.sh.
# - Mainnet needs KINLOCK_ALLOW_MAINNET=1 AND typing a confirmation phrase.
# - STELLAR_ACCOUNT is the NAME of an identity in your local Stellar CLI keystore, never a key.
# - The admin is set by the constructor at deploy time (ADR-0018), so it can't be front-run.
# - Writes deployments/<network>.json and regenerates DEPLOYMENTS.md. Never edit either by hand.
set -euo pipefail

cd "$(dirname "$0")/.."

DRY_RUN=0
case "${1:-}" in
  "") ;;
  --dry-run) DRY_RUN=1 ;;
  *) echo "usage: $0 [--dry-run]" >&2; exit 2 ;;
esac

die() { echo "deploy: $*" >&2; exit 1; }

NETWORK="${STELLAR_NETWORK:-testnet}"
case "$NETWORK" in
  testnet | local) ;;
  mainnet)
    [[ "${KINLOCK_ALLOW_MAINNET:-}" == "1" ]] ||
      die "refusing mainnet: set KINLOCK_ALLOW_MAINNET=1 and confirm interactively"
    read -r -p "Type 'deploy to mainnet' to continue: " answer
    [[ "$answer" == "deploy to mainnet" ]] || die "aborted"
    ;;
  *) die "unknown network '$NETWORK' (use testnet, local, or mainnet)" ;;
esac

[[ -n "${STELLAR_ACCOUNT:-}" ]] || die "set STELLAR_ACCOUNT to a Stellar CLI identity name"
[[ "${KINLOCK_ADMIN:-}" =~ ^[GC][A-Z2-7]{55}$ ]] ||
  die "set KINLOCK_ADMIN to the admin's G… or C… address"
command -v stellar >/dev/null || die "stellar CLI not found"
command -v jq >/dev/null || die "jq not found"

WASM="target/wasm32v1-none/release/kinlock.wasm"
echo "deploy: building" >&2
stellar contract build >&2
[[ -f "$WASM" ]] || die "build did not produce $WASM"
WASM_SHA256="$(sha256sum "$WASM" | cut -d' ' -f1)"

echo "deploy: network=$NETWORK account=$STELLAR_ACCOUNT admin=$KINLOCK_ADMIN wasm_sha256=$WASM_SHA256" >&2
if [[ $DRY_RUN -eq 1 ]]; then
  echo "deploy: dry run, nothing sent" >&2
  exit 0
fi

WASM_HASH="$(stellar contract upload --wasm "$WASM" \
  --source-account "$STELLAR_ACCOUNT" --network "$NETWORK")"
[[ "$WASM_HASH" =~ ^[0-9a-f]{64}$ ]] || die "unexpected wasm hash from upload: $WASM_HASH"
[[ "$WASM_HASH" == "$WASM_SHA256" ]] || die "uploaded hash $WASM_HASH != local sha256 $WASM_SHA256"

CONTRACT_ID="$(stellar contract deploy --wasm-hash "$WASM_HASH" \
  --source-account "$STELLAR_ACCOUNT" --network "$NETWORK" \
  -- --admin "$KINLOCK_ADMIN")"
[[ "$CONTRACT_ID" =~ ^C[A-Z2-7]{55}$ ]] || die "unexpected contract id from deploy: $CONTRACT_ID"

mkdir -p deployments
jq -n \
  --arg network "$NETWORK" \
  --arg contract_id "$CONTRACT_ID" \
  --arg wasm_hash "$WASM_HASH" \
  --arg admin "$KINLOCK_ADMIN" \
  --arg git_commit "$(git rev-parse HEAD)" \
  --arg git_dirty "$([[ -n "$(git status --porcelain)" ]] && echo true || echo false)" \
  --arg cli "$(stellar --version | head -1)" \
  --arg deployed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{network: $network, contract_id: $contract_id, wasm_hash: $wasm_hash, admin: $admin,
    git_commit: $git_commit, git_dirty: ($git_dirty == "true"), stellar_cli: $cli,
    deployed_at: $deployed_at}' \
  > "deployments/$NETWORK.json"

scripts/gen-deployments-md.sh
echo "deploy: $CONTRACT_ID on $NETWORK (recorded in deployments/$NETWORK.json)" >&2
echo "$CONTRACT_ID"
