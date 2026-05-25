# Tests

LiteSVM-based integration tests for the AMM program. Each test boots a fresh
in-process SVM, loads the compiled program from `target/deploy/amm_program.so`,
and exercises one instruction (or sequence of instructions) end-to-end.

## Layout

```
tests/
├── tests.rs                # one #[test] per scenario
└── ix_handlers/            # helpers that build each Instruction
    ├── mod.rs
    ├── init.rs
    ├── deposit.rs
    ├── withdraw.rs
    └── swap.rs
```

Splitting the instruction builders out of `tests.rs` keeps the actual test
bodies focused on setup → send → assert.

## Running

```bash
anchor build           # produces target/deploy/amm_program.so
cargo test --test tests
```

`tests.rs` reads the `.so` via `include_bytes!`, so an `anchor build` must run
before (or alongside) any code change to the program.

## Setup helper

`setup()` in [tests.rs](tests.rs) returns a fully wired tuple
`(svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y)`:

- new `LiteSVM` with the program loaded and the payer airdropped 1 SOL
- two fresh 6-decimal mints owned by the payer
- the `config` and `mint_lp` PDAs derived under seed `123`
- vault ATA addresses (the accounts themselves are created lazily by the
  `init` ix via `init_if_needed` / `associated_token::init`)

## Scenarios

| Test              | Builds                                | Asserts                                                              |
| ----------------- | ------------------------------------- | -------------------------------------------------------------------- |
| `test_initialize` | `init`                                | Both vaults and the LP mint exist with zero balance / supply.        |
| `test_deposit`    | `init` → `deposit`                    | Empty-pool deposit takes `max_x`/`max_y` verbatim and mints `amount`.|
| `test_withdraw`   | `init` → `deposit` → `withdraw`       | Burning 10M of 100M LP returns ~10% of each vault.                   |
| `test_swap`       | `init` → `deposit` → `swap`           | Receives ≥ `min` out, vault drains exactly that, and `k` doesn't fall. |

The deposit handler uses fixed numbers (`amount=100M`, `max_x=max_y=200M`,
user balance = 1B of each), and the withdraw / swap tests reuse those, so the
assertion magic numbers in `tests.rs` are all derived from those constants.

## `send` helper

`send` in [tests.rs](tests.rs) expires the blockhash, fetches the new one,
wraps the instructions in a legacy `VersionedTransaction`, and submits it.
Calling `expire_blockhash` every time is what lets multiple test cases reuse
the same `LiteSVM` instance without blockhash collisions.

## Adding a new test

1. Build the instruction in a new file under `ix_handlers/` (or extend an
   existing one) and re-export it from [ix_handlers/mod.rs](ix_handlers/mod.rs).
2. Add `#[test] fn test_…()` to [tests.rs](tests.rs): call `setup()`, build
   the instructions, send them via `send`, then assert on
   `token_balance` / `mint_supply`.
