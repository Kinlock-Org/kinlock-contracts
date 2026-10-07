#!/usr/bin/env bash
# Set up a ready-to-use Kinlock on the LOCAL quickstart network (roadmap M1-37), so anyone can
# run the SDK smoke test or develop against a fresh contract:
#
#   scripts/localnet.sh                 # start the local network (Docker)
#   scripts/localnet-setup.sh           # this script
#
# Creates and funds local identities (local-admin, local-attester, local-sender, local-payee,
# local-issuer) in your Stellar CLI keystore, issues a local "USDC" from local-issuer, gives the
# sender and payee trustlines and the sender 100 USDC, deploys the contract with local-admin as
# admin, adds the attester and the token, and registers one School payee whose payout is
# local-payee. Prints the env block for kinlock-sdk's `packages/sdk/scripts/smoke.mjs`.
#
# LOCAL ONLY: it refuses to run unless the RPC reports the standalone network passphrase.
# Identities are overwritten each run; they hold nothing outside this local network.
set -euo pipefail

cd "$(dirname "$0")/.."
die() { echo "localnet-setup: $*" >&2; exit 1; }
log() { echo "localnet-setup: $*" >&2; }

RPC_URL="${STELLAR_RPC_URL:-http://localhost:8000/rpc}"
LOCAL_PASSPHRASE="Standalone Network ; February 2017"
PAYEE_SLUG="${KINLOCK_LOCAL_PAYEE_SLUG:-local-test-school}"
SENDER_USDC_STROOPS=1000000000 # 100 USDC at 7 decimals

command -v stellar >/dev/null || die "stellar CLI not found"
command -v jq >/dev/null || die "jq not found"

passphrase="$(curl -sf -X POST "$RPC_URL" -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getNetwork"}' | jq -r .result.passphrase)" ||
  die "no RPC at $RPC_URL (start it with scripts/localnet.sh)"
[[ "$passphrase" == "$LOCAL_PASSPHRASE" ]] || die "refusing: $RPC_URL is '$passphrase', not the local network"

log "identities"
for name in local-admin local-attester local-sender local-payee local-issuer; do
  stellar keys generate "$name" --network local --fund --overwrite >/dev/null 2>&1 ||
    die "could not create and fund $name"
done
addr() { stellar keys address "$1"; }

log "local USDC"
ASSET="USDC:$(addr local-issuer)"
TOKEN="$(stellar contract asset deploy --asset "$ASSET" --source-account local-issuer \
  --network local 2>/dev/null | grep -oE '^C[A-Z2-7]{55}$' | tail -1)"
[[ "$TOKEN" =~ ^C[A-Z2-7]{55}$ ]] || die "token contract deploy failed"
for who in local-sender local-payee; do
  stellar tx new change-trust --source-account "$who" --line "$ASSET" --network local >/dev/null 2>&1 ||
    die "trustline for $who failed"
done
stellar tx new payment --source-account local-issuer --destination "$(addr local-sender)" \
  --asset "$ASSET" --amount "$SENDER_USDC_STROOPS" --network local >/dev/null 2>&1 ||
  die "funding local-sender failed"

log "contract"
stellar contract build >/dev/null 2>&1 || die "contract build failed (run: stellar contract build)"
CONTRACT_ID="$(stellar contract deploy --wasm target/wasm32v1-none/release/kinlock.wasm \
  --source-account local-admin --network local -- --admin "$(addr local-admin)" 2>/dev/null |
  grep -oE '^C[A-Z2-7]{55}$' | tail -1)"
[[ "$CONTRACT_ID" =~ ^C[A-Z2-7]{55}$ ]] || die "contract deploy failed"
invoke() { local who="$1"; shift; stellar contract invoke --id "$CONTRACT_ID" --network local --source-account "$who" -- "$@"; }
invoke local-admin add_attester --attester "$(addr local-attester)" >/dev/null 2>&1 || die "add_attester failed"
invoke local-admin add_token --token "$TOKEN" >/dev/null 2>&1 || die "add_token failed"

log "payee $PAYEE_SLUG"
PAYEE_ID="$(printf '%s' "$PAYEE_SLUG" | sha256sum | cut -d' ' -f1)"
# A placeholder meta_hash: there is no registry file for local payees.
META_HASH="$(printf 'kinlock-local:%s' "$PAYEE_SLUG" | sha256sum | cut -d' ' -f1)"
invoke local-attester register_payee --attester "$(addr local-attester)" --payee_id "$PAYEE_ID" \
  --payout "$(addr local-payee)" --meta_hash "$META_HASH" --category School >/dev/null 2>&1 ||
  die "register_payee failed"

balance="$(stellar contract invoke --id "$TOKEN" --network local --source-account local-sender \
  --send=no -- balance --id "$(addr local-sender)" 2>/dev/null | tr -d '"')"
[[ "$balance" == "$SENDER_USDC_STROOPS" ]] || die "sender balance is $balance, expected $SENDER_USDC_STROOPS"

log "done"
cat <<ENV
# Local Kinlock ($(date -u +%Y-%m-%dT%H:%M:%SZ)). For kinlock-sdk packages/sdk/scripts/smoke.mjs,
# which defaults to http://localhost:8000/rpc on local. (Don't export STELLAR_RPC_URL here: the
# stellar CLI reads it too and then rejects --network local.)
export KINLOCK_NETWORK=local
export KINLOCK_CONTRACT_ID=$CONTRACT_ID
export KINLOCK_TOKEN=$TOKEN
export KINLOCK_PAYEE_ID=$PAYEE_ID
export KINLOCK_SENDER_IDENTITY=local-sender
export KINLOCK_PAYEE_IDENTITY=local-payee
export KINLOCK_INDEXER_URL=
ENV
