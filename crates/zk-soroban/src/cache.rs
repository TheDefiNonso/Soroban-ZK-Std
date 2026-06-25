//! Instance-storage caching of recurring BN254 cryptographic constants (Issue #124).
//!
//! The BN254 Poseidon2 permutation depends on a fixed matrix diagonal and 64
//! round-constant rows, and field arithmetic depends on the Fr modulus. These
//! values are deterministic and identical on every contract invocation, yet the
//! pure builders in [`crate::poseidon2`] rebuild them from code on each call.
//!
//! This module caches them in the contract's `StorageType::Instance` using lazy
//! initialisation: the first read within a contract computes the constant and
//! writes it to instance storage; later reads return the stored copy. The
//! instance TTL is bumped on every access so the cache stays live for as long
//! as the contract is in active use.
//!
//! ## Security
//! Instance storage is owned exclusively by the contract — external callers
//! cannot write to it — so a cached constant cannot be tampered with by a
//! third party. Because every value is also fully recomputable from code, a
//! cache miss (for example after TTL expiry) is recovered transparently with
//! no loss of correctness.

use soroban_sdk::{contracttype, Env, Vec, U256};

/// Lower TTL bound (in ledgers) for the instance entry. When the remaining
/// time-to-live drops below this threshold, the entry is extended back up to
/// [`INSTANCE_BUMP_AMOUNT`]. ~1 day at 5s ledger close time.
const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;

/// Target TTL (in ledgers) the instance entry is extended to on access.
/// ~30 days at 5s ledger close time.
const INSTANCE_BUMP_AMOUNT: u32 = 518_400;

/// Keys for the recurring cryptographic constants cached in instance storage.
#[contracttype]
#[derive(Clone)]
pub enum ConstantKey {
    /// BN254 Poseidon2 (t=3) 64 round-constant rows.
    Poseidon2RoundConstants,
    /// BN254 Poseidon2 internal matrix diagonal (M_I − I) = [1, 1, 2].
    Poseidon2MatDiag,
    /// BN254 Fr field modulus r.
    FrModulus,
}

/// Bump the instance TTL so the cached constants stay live during active use.
fn bump(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Return the cached BN254 Poseidon2 round constants, computing and storing
/// them on the first call within the contract.
pub fn round_constants(env: &Env) -> Vec<Vec<U256>> {
    let store = env.storage().instance();
    let value = match store.get(&ConstantKey::Poseidon2RoundConstants) {
        Some(rc) => rc,
        None => {
            let rc = crate::poseidon2::round_constants(env);
            store.set(&ConstantKey::Poseidon2RoundConstants, &rc);
            rc
        }
    };
    bump(env);
    value
}

/// Return the cached BN254 Poseidon2 matrix diagonal, computing and storing it
/// on the first call within the contract.
pub fn mat_diag(env: &Env) -> Vec<U256> {
    let store = env.storage().instance();
    let value = match store.get(&ConstantKey::Poseidon2MatDiag) {
        Some(mat) => mat,
        None => {
            let mat = crate::poseidon2::mat_diag(env);
            store.set(&ConstantKey::Poseidon2MatDiag, &mat);
            mat
        }
    };
    bump(env);
    value
}

/// Return the cached BN254 Fr modulus, computing and storing it on the first
/// call within the contract.
pub fn fr_modulus(env: &Env) -> U256 {
    let store = env.storage().instance();
    let value = match store.get(&ConstantKey::FrModulus) {
        Some(m) => m,
        None => {
            let m = crate::poseidon2::fr_modulus(env);
            store.set(&ConstantKey::FrModulus, &m);
            m
        }
    };
    bump(env);
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZkContract;
    use soroban_sdk::Env;

    fn env() -> Env {
        let e = Env::default();
        e.cost_estimate().budget().reset_unlimited();
        e
    }

    #[test]
    fn round_constants_lazy_init_then_cache_hit() {
        let env = env();
        let id = env.register(ZkContract, ());
        env.as_contract(&id, || {
            let store = env.storage().instance();
            // Cold: nothing cached yet.
            assert!(!store.has(&ConstantKey::Poseidon2RoundConstants));

            // First read populates the cache and matches the pure builder.
            let first = round_constants(&env);
            assert!(store.has(&ConstantKey::Poseidon2RoundConstants));
            assert_eq!(first, crate::poseidon2::round_constants(&env));

            // Second read is a cache hit returning identical data.
            let second = round_constants(&env);
            assert_eq!(first, second);
        });
    }

    #[test]
    fn mat_diag_and_modulus_are_cached() {
        let env = env();
        let id = env.register(ZkContract, ());
        env.as_contract(&id, || {
            assert_eq!(mat_diag(&env), crate::poseidon2::mat_diag(&env));
            assert_eq!(fr_modulus(&env), crate::poseidon2::fr_modulus(&env));

            let store = env.storage().instance();
            assert!(store.has(&ConstantKey::Poseidon2MatDiag));
            assert!(store.has(&ConstantKey::FrModulus));
        });
    }
}
