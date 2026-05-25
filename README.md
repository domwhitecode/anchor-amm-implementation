# AMM Program

A constant-product (`x * y = k`) automated market maker built with Anchor for
Solana. Pools hold two SPL tokens (`X` and `Y`), liquidity providers mint and
burn LP tokens, and swappers trade against the pool subject to a configurable
basis-point fee.

The curve math is delegated to
[`constant-product-curve`](https://github.com/deanmlittle/constant-product-curve).

- **Program ID:** `8Wrtbhw9tKyTW7Z1oMyQrhivEoLZCPRxVim1ey9wvTiq`
- **Anchor cluster:** `localnet` (see [Anchor.toml](Anchor.toml))

## Layout

```
programs/amm-program/
├── src/
│   ├── lib.rs            # #[program] entrypoint — wires ix to handlers
│   ├── constants.rs      # PDA seed constants (CONFIG_SEED, LP_SEED)
│   ├── error.rs          # AmmError + CurveError → AmmError mapping
│   ├── state.rs          # Config account
│   └── instructions/     # see programs/amm-program/src/instructions/README.md
└── tests/                # see programs/amm-program/tests/README.md
```

## Account model

Each pool is keyed by a `u64` seed and owns:

| Account   | Derivation                                             | Purpose                      |
| --------- | ------------------------------------------------------ | ---------------------------- |
| `config`  | `[CONFIG_SEED, seed.to_le_bytes()]`                    | Pool state (fee, mints, …)   |
| `mint_lp` | `[LP_SEED, config.key()]`                              | LP token mint (6 decimals)   |
| `vault_x` | ATA of `config` for `mint_x`                           | Pool's X reserves            |
| `vault_y` | ATA of `config` for `mint_y`                           | Pool's Y reserves            |

`config` is the authority for `mint_lp` and both vaults — token CPIs from the
program sign with the config PDA seeds.

## Instructions

| Ix           | Signers | Effect                                                                                    |
| ------------ | ------- | ----------------------------------------------------------------------------------------- |
| `initialize` | maker   | Creates `config`, `mint_lp`, and both vaults for a new pool.                              |
| `deposit`    | user    | Pulls X and Y from the user, mints LP. First deposit seeds the curve verbatim.            |
| `withdraw`   | user    | Burns LP, returns proportional X and Y. Slippage-protected by `min_x` / `min_y`.          |
| `swap`       | user    | Trades exact-in `X→Y` or `Y→X` with `min`-out slippage protection; fees stay in the pool. |

See [programs/amm-program/src/instructions/README.md](programs/amm-program/src/instructions/README.md)
for argument and account details.

## Build & test

```bash
anchor build           # compiles the BPF .so into target/deploy/
cargo test --test tests   # runs the LiteSVM integration tests
```

`cargo test` requires `anchor build` to have produced
`target/deploy/amm_program.so` first — the test harness loads it via
`include_bytes!`.
