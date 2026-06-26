# Soroban-ZK-Std

**A High-Performance Cryptographic Standard Library for Stellar Protocol 25 ZK-Primitives.**

This SDK provides a ready-to-use toolkit for Stellar smart contract developers to verify zero-knowledge proofs (such as Groth16) on-chain using Soroban. It bridges the gap between the low-level Protocol 25 BN254 host functions and the high-level needs of modern privacy applications.

## Installation

Add the library to your Soroban smart contract's `Cargo.toml`:

```toml
[dependencies]
soroban-zk-std = "0.1.3"
```

## Quick Start: Verifying a Groth16 Proof

This library provides a highly optimized, single-call 4-pairing check for Groth16 verifiers using the native `bn254_multi_pairing_check` host function.

Here is a simple example of how to verify a Groth16 proof inside your contract:

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};
use soroban_zk_std::groth16::{groth16_verify, Groth16Proof, Groth16VerifyingKey};
use ethnum::u256;

#[contract]
pub struct ZKVerifierContract;

#[contractimpl]
impl ZKVerifierContract {
    pub fn verify_proof(env: Env, proof_bytes: Bytes, public_input_bytes: Bytes) -> bool {
        // 1. Deserialize the Groth16 Proof (A, B, C points)
        let mut proof_buf = [0u8; 256];
        proof_bytes.copy_into_slice(&mut proof_buf);
        let proof = Groth16Proof::from_bytes(&proof_buf).expect("Invalid proof format");

        // 2. Load your circuit's Verifying Key
        let vk = get_verifying_key(); // Fetch from storage or hardcode

        // 3. Parse public inputs
        let mut pi_buf = [0u8; 32];
        public_input_bytes.copy_into_slice(&mut pi_buf);
        let public_input = u256::from_be_bytes(pi_buf);

        // 4. Verify the Zero Knowledge Proof!
        groth16_verify(&env, &vk, &proof, &[public_input]).unwrap_or(false)
    }
}
```

## Features

- **Host-Guest Mapping**: Seamlessly converts between Soroban's `U256` and the internal BN254 field representations.
- **Gas Efficient**: Wraps native Stellar Protocol 25 primitives to keep instruction counts incredibly low.
- **Constant Time**: Ensures cryptographic operations are side-channel resistant.
- **No-Std**: Fully compatible with the `#![no_std]` environment required by Soroban.

## Learn More

For complete documentation, contributing guidelines, and more examples (like our Shielded Asset Template), please visit the [Main Repository on GitHub](https://github.com/georgegoldman/Soroban-ZK-Std).
