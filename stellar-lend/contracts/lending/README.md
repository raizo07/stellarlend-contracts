# StellarLend Lending Contract

A secure, efficient lending protocol built on Soroban that allows users to borrow assets against collateral.

## Features

- **Collateralized Borrowing**: Borrow assets by providing collateral with a minimum 150% ratio
- **Interest Accrual**: Automatic interest calculation at 5% APY
- **Debt Ceiling**: Protocol-level debt limits for risk management
- **Collateralized Borrowing**: Borrow assets by providing collateral with a minimum 150% ratio
- **Interest Accrual**: Automatic interest calculation at 5% APY
- **Debt Ceiling**: Protocol-level debt limits for risk management
- **Pause Mechanism**: Granular emergency pause functionality for specific operations (Deposit, Borrow, Repay, Withdraw, Liquidation)
- **Emergency Lifecycle**: `Normal -> Shutdown -> Recovery -> Normal` flow with guardian-triggered shutdown and admin-controlled recovery
- **Admin Control**: Secure protocol management with a dedicated admin role
- **Overflow Protection**: Comprehensive checks against arithmetic overflow
- **Event Emission**: Track all borrow and pause operations via events

## Building

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Testing

```bash
cargo test
```

## Documentation

See [borrow.md](./borrow.md), [pause.md](./pause.md), and [emergency_shutdown.md](./emergency_shutdown.md) for comprehensive documentation including:

- Function signatures and parameters
- Error types and handling
- Security assumptions
- Usage examples
- Best practices

## View Serialization Stability

The contract treats the current struct-returning getter responses as view schema `v1`.

Covered getters:

- `get_user_debt() -> DebtPosition`
- `get_user_collateral() -> BorrowCollateral`
- `get_user_collateral_deposit() -> DepositCollateral`
- `get_user_position() -> UserPositionSummary`

Wire-format guarantee:

- Soroban `#[contracttype]` structs encode as XDR maps keyed by field name.
- The generated conversion code sorts those keys lexicographically, so the on-wire key order is deterministic.
- Snapshot-style tests lock the current XDR encoding for the getter structs above.

Stable decoding guidance:

- Decode these responses by field name, not by source declaration order.
- Treat the current field set and field names as schema `v1`.
- Do not assume a new field can be added safely to an existing getter response. Even additive changes can break strict decoders and hash-based snapshots.

Versioning strategy:

- Existing getter response structs are preserved in place for schema `v1`.
- Any additive or breaking change to one of the getter structs must ship as a new versioned getter/type, for example `get_user_position_v2()`, instead of mutating the current response shape.
- A runtime `schema` field is intentionally not added to the existing structs because that would itself be a breaking ABI change for the current getter surface.

## Contract Interface

### Main Functions

- `borrow()` - Borrow assets against collateral
- `get_user_debt()` - Query user's debt position
- `get_user_collateral()` - Query user's collateral position

### Admin Functions

- `initialize()` - Set protocol admin, debt ceiling, and minimum borrow amount
- `set_pause()` - Granular pause for specific operations
- `set_guardian()` - Configure emergency guardian
- `emergency_shutdown()` - Trigger hard emergency shutdown (admin or guardian)
- `start_recovery()` - Enter controlled user unwind mode (admin only)
- `complete_recovery()` - Return protocol to normal operation (admin only)
- `get_admin()` - Returns the current protocol admin

## Security

- Minimum 150% collateral ratio enforced
- All arithmetic operations use checked methods
- Authorization required for all user operations
- Comprehensive test coverage including edge cases

## License

See repository root for license information.
