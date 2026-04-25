# Security Notes & Trust Boundaries

## Trust Boundaries
- **Admins:** The highest level of privilege. Admins can update parameters (such as minimum borrow amounts, deposit ceilings, and oracles), pause the protocol, trigger emergency shutdown, and designate guardians. They are also responsible for upgrading the protocol.
- **Guardians:** Designed for rapid response. Guardians can only trigger emergency shutdowns. They cannot upgrade contracts, unpause the system, or change parameters.
- **Users:** End-users interact with the protocol via `deposit`, `borrow`, `repay`, and `withdraw` mechanisms subject to protocol checks. User operations are sandboxed to their respective `Address` scopes.
- **Oracles:** Trusted entities providing price feeds used for health factor checks. If an oracle becomes malicious, it could trigger improper liquidations, but internal checks restrict maximum liquidation amounts (via close factor limits).

## Authorization Model
All external entry points modifying state or user balances call `user.require_auth()`. This delegates authorization entirely to the Soroban SDK's robust authorization framework. 
Protocol functions restricted to Admins enforce validation via `admin.require_auth()` and ensure the caller matches the registered Admin in the data store.

## Reentrancy Protections
In Soroban, contract logic guarantees atomicity. However, as an added measure against logic-based reentrancy across cross-contract calls:
- All external calls to update state (e.g. `save_deposit_position`) occur *before* external token transfers where applicable (the Checks-Effects-Interactions pattern).
- High-risk operations are guarded by global pause mappings which an Admin or Guardian can engage via the pause module if anomalous behavior occurs.

## Cross-Asset Module Hardening
- **Token Transfer Enforcement:** All position operations (`deposit`, `borrow`, `repay`, `withdraw`) now explicitly enforce token transfers via the Soroban `token::Client`.
- **Granular Pause Support:** Cross-asset operations now respect specific `PauseType` settings (e.g. `PauseType::Borrow`), allowing for targeted emergency interventions.
- **Event-Driven Transparency:** Each significant operation emits a unique contract event (`CrossDepositEvent`, etc.), facilitating robust off-chain monitoring and audit trails.
- **Initialization Safety:** The `initialize_admin` function now returns a `Result` and prevents re-initialization if an admin is already set.

## Arithmetic Bounds
Protocol parameters strictly utilize `checked_add`, `checked_sub`, `checked_mul`, and `checked_div` to prevent overflow and underflow paths. Zero-amount and uninitialized parameter paths intentionally return structured `ContractError` values rather than panicking where possible.

## Withdraw path (`withdraw.rs`)
- **Pause module**: Withdraw is blocked when `pause::is_paused(Withdraw)` is true (this includes global `PauseType::All`), when the legacy `WithdrawDataKey::Paused` flag is set, or when the protocol is in **emergency shutdown** (`blocks_high_risk_ops` and not in **recovery**). In **recovery**, users may still withdraw (and repay) to unwind positions.
- **Collateral ratio**: Post-withdraw collateral must satisfy the same minimum ratio as borrows, via shared `borrow::validate_collateral_ratio` (150% default, `MIN_COLLATERAL_RATIO_BPS`).
- **Authorization**: Only the position owner can withdraw; `user.require_auth()` is enforced before state changes.

## Borrow-Withdraw Invariant (Security Boundary)

The protocol enforces a critical invariant: **after every successful `withdraw`, the remaining collateral must continue to satisfy the minimum collateral ratio against the user's total debt** (including accrued interest). This prevents a class of exploits where a borrower attempts to withdraw collateral immediately after borrowing, or after a partial repay, in order to leave their position undercollateralized.

### Prevented exploit classes

| Exploit attempt | Defence mechanism |
|---|---|
| Borrow then immediately withdraw all collateral | `validate_collateral_ratio_after_withdraw` rejects because remaining collateral < required |
| Borrow at exactly 150 % boundary, then withdraw 1 unit | Same ratio check; 1 unit below boundary fails |
| Rounding manipulation with small amounts | Integer math is exact; no rounding down in borrow's favour |
| Interest accrual timing attack (wait, then withdraw before interest "counts") | Interest is calculated fresh on every withdraw call using current timestamp |
| Partial repay, then over-withdraw | Repay reduces debt; withdraw re-evaluates required collateral from current debt |
| View inconsistency (health factor says liquidatable, but withdraw succeeds) | Withdraw uses the **borrow** ratio (150 %), not the liquidation threshold (80 %), so it is stricter |
| Rapid borrow-withdraw cycles to drain collateral | Each withdraw independently re-validates the ratio; cumulative debt is tracked |
| Deposit, borrow, withdraw original deposit | Borrow collateral and deposit collateral share the same balance; total is checked |
| Oracle price drop makes HF < 1.0 | Withdraw still enforces 150 % raw collateral ratio regardless of oracle price |

### Implementation note

`withdraw::validate_collateral_ratio_after_withdraw` delegates directly to `borrow::validate_collateral_ratio`. Using the **same function** for both borrow-time and withdraw-time validation guarantees that the two paths can never drift out of agreement. Any future change to the collateral ratio rule automatically applies to both entry points.
