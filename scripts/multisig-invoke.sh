#!/usr/bin/env bash
# [sec] Call a contract function as a multisig account (e.g. the testnet admin).
#
#   STELLAR_NETWORK=testnet scripts/multisig-invoke.sh <source-identity> <signer-identity>... \
#     -- <function> [--arg value ...]
#
# Builds the call with the multisig account as transaction source, simulates it, appends one
# signature per signer identity, and sends it. Soroban accepts the call only if the signatures
# meet the account's medium threshold. The contract ID comes from deployments/<network>.json.
# Mainnet is refused: mainnet signing belongs on hardware keys (roadmap H-15).
set -euo pipefail

cd "$(dirname "$0")/.."
die() { echo "multisig-invoke: $*" >&2; exit 1; }

NETWORK="${STELLAR_NETWORK:-testnet}"
case "$NETWORK" in
  testnet | local) ;;
  *) die "refusing network '$NETWORK' (testnet or local only)" ;;
esac

[[ $# -ge 3 ]] || die "usage: $0 <source-identity> <signer-identity>... -- <function> [args]"
SOURCE="$1"; shift
SIGNERS=()
while [[ $# -gt 0 && "$1" != "--" ]]; do SIGNERS+=("$1"); shift; done
[[ "${1:-}" == "--" ]] || die "missing -- before the function call"
shift
[[ ${#SIGNERS[@]} -ge 1 && $# -ge 1 ]] || die "need at least one signer and a function"

CONTRACT_ID="$(jq -r .contract_id "deployments/$NETWORK.json")"
[[ "$CONTRACT_ID" =~ ^C[A-Z2-7]{55}$ ]] || die "no contract id in deployments/$NETWORK.json"

TX="$(stellar contract invoke --id "$CONTRACT_ID" --network "$NETWORK" \
  --source-account "$SOURCE" --build-only -- "$@")"
TX="$(echo "$TX" | stellar tx simulate --network "$NETWORK" --source-account "$SOURCE")"
for signer in "${SIGNERS[@]}"; do
  TX="$(echo "$TX" | stellar tx sign --sign-with-key "$signer" --network "$NETWORK")"
done
echo "$TX" | stellar tx send --network "$NETWORK"
