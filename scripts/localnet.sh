#!/usr/bin/env bash
# Start a local Stellar quickstart network (node, RPC, friendbot) in Docker.
set -euo pipefail
exec stellar container start local "$@"
