# Instructions

Per-instruction handlers for the AMM. Each file defines an `#[derive(Accounts)]`
context plus an `impl` that performs the business logic. The `#[program]` module
in [../lib.rs](../lib.rs) is a thin wrapper that forwards into these `impl`s.

## `initialize` — [initialize.rs](initialize.rs)

Creates a new pool.

**Args:** `seed: u64`, `fee: u16` (basis points, 0–10_000), `authority: Option<Pubkey>`,
`config_bump: u8`, `mint_lp_bump: u8`.

Initializes `config`, the LP mint (6 decimals, authority = `config`), and the
two vault ATAs (`vault_x`, `vault_y`, both owned by `config`). The `seed`
parameter lets a single initializer create multiple distinct pools.

## `deposit` — [deposit.rs](deposit.rs)

Adds liquidity and mints LP tokens to the user.

**Args:** `amount: u64` (LP to mint), `max_x: u64`, `max_y: u64` (slippage caps).

Two code paths:

- **Empty pool** (`mint_lp.supply == 0 && vaults both empty`): deposits exactly
  `max_x` / `max_y` and mints `amount` LP — the depositor seeds both the
  reserves and the LP supply.
- **Existing pool:** calls `ConstantProduct::xy_deposit_amounts_from_l` to
  derive the required X / Y for `amount` LP at the current ratio. Reverts with
  `SlippageExceeded` if either exceeds the caller's cap.

Reverts with `PoolLocked` if `config.locked` is set, or `InvalidAmount` if
`amount == 0`.

## `withdraw` — [withdraw.rs](withdraw.rs)

Burns LP tokens and returns the proportional share of each vault.

**Args:** `amount: u64` (LP to burn), `min_x: u64`, `min_y: u64` (slippage floors).

Calls `ConstantProduct::xy_withdraw_amounts_from_l` to compute the X / Y to
release, reverts with `SlippageExceeded` if either is below the floor, then
burns LP first and transfers the tokens out from the vaults (signed by the
`config` PDA).

## `swap` — [swap.rs](swap.rs)

Exact-in swap of one side of the pair for the other.

**Args:** `is_x: bool` (true ⇒ deposit X, withdraw Y), `amount: u64` (exact in),
`min: u64` (slippage floor on the out side).

Initializes a `ConstantProduct` with the live vault balances, LP supply, and
the pool's fee (applied internally by the curve), then performs the swap.
Fees stay in the pool, so `k` is non-decreasing.

## Curve precision gotcha

`ConstantProduct::init` takes the number of decimals (`Some(6)` → `10^6`
internally), but the **static** helpers
`xy_{deposit,withdraw}_amounts_from_l` take the *raw multiplier* (`1_000_000`),
not the exponent. Passing the exponent (e.g. `6`) silently destroys precision
and the proportional payout collapses to ~`a/precision` per unit. Both call
sites in `deposit.rs` and `withdraw.rs` pass `1_000_000`.

## Shared conventions

- All cross-program token operations that move funds out of a vault sign with:

  ```rust
  &[&[CONFIG_SEED, &config.seed.to_le_bytes(), &[config.config_bump]]]
  ```

  because the vault authority is the `config` PDA. LP burns are signed by the
  user (the LP ATA's authority).

- Errors are defined in [../error.rs](../error.rs); `CurveError`s from the
  curve crate are mapped into the program's `AmmError` enum via `From`.
