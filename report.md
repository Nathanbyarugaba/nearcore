# Defensive Security Research: Consensus Determinism & Safety

## Codebase Map
- **Consensus Engine & Block Validation**: `chain/chain/src` (specifically `chain.rs`, `doomslug.rs`, `validate.rs`, `signature_verification.rs`)
- **Transaction Execution & VM**: `runtime/runtime/src` (specifically `actions.rs`, `action_validation.rs`, `receipt_manager.rs`, `verifier.rs`, `function_call.rs`)
- **Serialization, Hashing & State Roots**: `core/primitives/src` (specifically `types.rs`, `hash.rs`, `receipt.rs`), `core/crypto/src`
- **Mempool-to-Block Boundaries**: `chain/pool/src` (mempool), `chain/chain/src/pending.rs`
- **Database & State Iteration**: `core/store/src`, `core/primitives/src`
- **Upgrade/Migration**: `core/primitives/src/epoch_manager.rs` (protocol versions)
- **Networking Assumptions**: `chain/network/src` (validation purely based on local determinism rules)

## Consensus-Safety Assumptions
1. **Exact determinism required**: applying chunk to the same pre-state must yield the exact same post-state.
2. **Deterministic Iteration**: Collections iteration over non-deterministic structures like `HashMap` or `HashSet` must be avoided if iteration order influences consensus/state-root. Most internal usage of `HashMap` observed (e.g. `promise_yield_receipt_index` in `receipt_manager.rs` and `shards_congestion_info` in `congestion_control.rs`) is for fast lookups. We verified `BlockCongestionInfo` wraps a `BTreeMap` correctly for iteration in `core/primitives/src/congestion_info.rs`. However, `ValidatorAccountsUpdate.stake_info` and `validator_rewards` use `HashMap` in `runtime/runtime/src/lib.rs` and iterators are used over them during `update_validator_accounts()`. This should ideally be sorted or use `BTreeMap` to avoid implicit non-deterministic state-root/DB commit order.
3. **Randomness**: Randomness must strictly stem from the block header (`random_value()`). No usage of local PRNGs (e.g. `thread_rng()`) during execution. Verified `thread_rng()` is scoped to tests only.

## High-Risk Modules
- `runtime/runtime/src/lib.rs` (`update_validator_accounts` iterating over `HashMap`).
- `runtime/runtime/src/congestion_control.rs` and `bandwidth_scheduler` (usage of maps for lookup is generally safe, but iteration must be rigorously monitored).

## Findings & Recommended Fixes
1. Usage of `HashMap` in `ValidatorAccountsUpdate` (`core/primitives/src/epoch_info.rs` and `runtime/runtime/src/lib.rs`). Iterating over a `HashMap` in `update_validator_accounts` might write to the Trie in a non-deterministic order. Even if the Trie absorbs order differences in memory, if state changes are emitted sequentially to a log or if out-of-memory commits happen, this could lead to determinism bugs. **Recommendation**: Migrate `ValidatorAccountsUpdate` to use `BTreeMap` or iterate over a sorted vector.
2. Random seed generation strictly relies on `block_header.random_value()`. Usage of `rand::thread_rng()` is correctly constrained to tests (`chain/chain/src/test_utils.rs`, `tests/garbage_collection.rs`).
3. Gas distributions and receipt forwarding iteration must remain deterministic. It uses `Vec` and `BTreeMap` reliably.

## Tests Added / Invariants Checked
- Explored codebase to verify that sources of non-determinism (thread scheduling, randomness, clock time) are successfully abstracted out of consensus-critical execution paths (`runtime/runtime/src/lib.rs`).
- Checked invariants on `HashMap` iteration: `ValidatorAccountsUpdate` is the primary offender that violates the rule of thumb "never iterate over `HashMap` in consensus code." A defensive test checking this invariant was added to `runtime/runtime/tests/determinism.rs`.
