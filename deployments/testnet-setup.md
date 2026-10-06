# Testnet setup

How the current testnet contract is configured (roadmap M1-19). The contract ID, WASM hash, and
deploy commit are in [`../DEPLOYMENTS.md`](../DEPLOYMENTS.md) (generated). This file is written by
hand; update it whenever the testnet configuration changes.

Contract: `CCSHDQFRYFC3AHV5NE6ULQW6X2CMG5RPANBORDXJGSUD6UKECASJQBRI` (deployed 2026-10-06).

## Admin: 2-of-3 multisig

| Role | Identity (local Stellar CLI keystore) | Address | Weight |
|---|---|---|---|
| Admin account | `kinlock-testnet-admin` | `GDTJN6RPBRSW4XLX7GHFSTRZJA7PRJUCI5DZLZ3LL2Y24EOJVQ4E7SHB` | 0 (own key disabled) |
| Signer | `kinlock-testnet-signer-1` | `GD3JQARGPOPBPRLRAP3ULSV2HAGJLCCOU5IGULYY2MHKIU2CZ3PQU6YZ` | 1 |
| Signer | `kinlock-testnet-signer-2` | `GCTPR24T6U7RXMFB44DSEWQMMLSHIUCQNVER6FJZMIZBITUCCOOKWYR7` | 1 |
| Signer | `kinlock-testnet-signer-3` | `GDURV6L3ZEUGNV4OFIHSBGQJAUPMDGZJMIH2WGPEM4WHOLNJYCOEPBRU` | 1 |

Thresholds low / medium / high = 2 / 2 / 2. Soroban checks a classic account's authorization
against its **medium** threshold, so every admin call needs two of the three signers.

Admin calls go through `scripts/multisig-invoke.sh`, for example:

```
scripts/multisig-invoke.sh kinlock-testnet-admin kinlock-testnet-signer-1 kinlock-testnet-signer-2 \
  -- set_paused_new_locks --paused true
```

These are **testnet-only** keys in the CLI's plain-file keystore. Mainnet admin keys must be
hardware keys from a key ceremony (roadmap H-15), never this keystore.

## Configuration

| Setting | Value | Transaction |
|---|---|---|
| Attester | `kinlock-testnet-attester-1` `GCZRNEOGZH7FVJTA4OQN2YQ5LMS7LSV2GP7PWJFC73SMSLPA56DRAPQT` | `9d89abefcc8d8fa4e826b107996d31317cb4e7b03631535ccb98d9ca1479578b` (2 signatures) |
| Token allowlist | Circle testnet USDC, issuer `GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5`, SAC `CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA` | `00f36eef368ad2ddd71eb59cd188d6826f41a2b977292be14332b4be79afc2d0` (2 signatures) |
| Paused | no | — |
| Caps | none set (unlimited) | — |
| Payees | none yet: they're added through the registry process | — |

The USDC issuer comes from Circle's published addresses
(developers.circle.com/stablecoins/usdc-contract-addresses); the SAC reports its name as
`USDC:GBBD47IF…FLA5` with 7 decimals.

## Checks run on 2026-10-06

- `add_attester` and `add_token` signed by **one** signer: rejected with `TxBadAuth`, nothing
  changed. Signed by **two**: accepted (both on-chain transactions carry 2 signatures).
- Simulated `register_payee` signed by the attester: succeeds, so the attester is on the roster.
- Simulated `create_lock` with USDC fails at the payee check (`#11 PayeeNotFound`); with an
  unlisted token it fails at the token check (`#21 TokenNotAllowed`). USDC is on the allowlist.

## Superseded

`CDIPDHSAKP2MNANLYV6VRWVTWFJH3PQRHBRDBVMYNMRYKSR66JVPTLYL` (2026-10-06, single-key admin
`kinlock-testnet-deployer`). The contract has no way to change its admin, so moving to the
multisig admin meant a new deployment. The old contract holds no funds and is unused.
