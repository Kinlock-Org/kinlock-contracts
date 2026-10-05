#!/usr/bin/env bash
# [sec] Deploy the kinlock contract. Defaults to testnet.
# Mainnet requires KINLOCK_ALLOW_MAINNET=1 AND an interactive confirmation.
# Writes deployments/<network>.json; DEPLOYMENTS.md is generated from it (never hand-edit).
#
# SCAFFOLD: guards only. The deploy steps land in M1-17 (ask-first: deploy scripts).
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"

if [[ "$NETWORK" == "mainnet" ]]; then
  if [[ "${KINLOCK_ALLOW_MAINNET:-}" != "1" ]]; then
    echo "Refusing mainnet: set KINLOCK_ALLOW_MAINNET=1 and confirm interactively." >&2
    exit 1
  fi
  read -r -p "Type 'deploy to mainnet' to continue: " answer
  [[ "$answer" == "deploy to mainnet" ]] || { echo "Aborted." >&2; exit 1; }
fi

echo "deploy.sh is not implemented yet (roadmap M1-17). Network would be: $NETWORK" >&2
exit 1
