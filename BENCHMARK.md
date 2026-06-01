# Soroban-ZK-Std Gas Benchmarks

This suite measures the CPU instruction cost of zero-knowledge cryptographic
primitives within the Soroban VM using the `soroban-sdk` budget API.

Soroban enforces a strict computational budget of **400,000,000 instructions**
per transaction. The results below confirm that a complete Groth16 verification
(including pairing and public-input accumulation) fits well within this limit.

> For the full cost reference including wall-clock Criterion data and
> transaction planning tables, see [GAS.md](./GAS.md).

## Environment

| Property            | Value                            |
|---------------------|----------------------------------|
| Soroban SDK version | 25.3.0                           |
| Stellar Protocol    | 25                               |
| Target              | `wasm32-unknown-unknown`         |
| Measurement API     | `env.cost_estimate().budget().cpu_instruction_cost()` |
| Profile             | `release` (`opt-level = z`, `lto = true`) |

## Results

### Poseidon2 Hashing (host-function — Soroban measured)

`hash_to_field` delegates to `env.crypto_hazmat().poseidon2_permutation()`.

| Operation          | Inputs | CPU Instructions | % of 400 M budget |
|--------------------|-------:|-----------------:|:-----------------:|
| `poseidon2_hash`   | 1      | 1,007,753        | 0.25%             |
| `poseidon2_hash`   | 2      | 2,010,994        | 0.50%             |
| `poseidon2_hash`   | 4      | 3,024,708        | 0.75%             |

### BN254 Field & Curve Arithmetic (WASM guest — wall-clock estimate)

Operations implemented natively in `zk-core` run as WASM guest code.
The Soroban SDK mock environment does not meter guest WASM instructions,
so these figures use Criterion wall-clock timings × 3,000 instructions/µs.

| Operation          | Est. CPU Instructions | % of 100 M budget |
|--------------------|----------------------:|:-----------------:|
| `Fr::add`          | ~300                  | < 0.001%          |
| `Fr::mul`          | ~669                  | < 0.001%          |
| `Fr::invert`       | ~5,550,000            | 5.55%             |
| `Fq::mul`          | ~4,620                | 0.005%            |
| `Fq::invert`       | ~5,790,000            | 5.79%             |
| `G1::scalar_mul`   | ~69,000,000           | 69.0%             |
| `G1::msm` (n=2)    | ~138,140,000          | 138% †            |
| `G1::msm` (n=4)    | ~276,210,000          | 276% †            |
| `G1::msm` (n=8)    | ~552,350,000          | 552% †            |

† Exceeds the per-operation 100 M limit; split across multiple transactions.

### Groth16 Verification (composite — 1 public input)

Combines an MSM(2) WASM guest cost with a 4-pair native pairing check.

| Component            | CPU Instructions | Source      |
|----------------------|-----------------:|:-----------:|
| MSM(2) accumulator   | ~138,140,000     | wall-clock  |
| 4-pair pairing check | 29,327,515       | Soroban API |
| **Total estimate**   | **~167,467,515** |             |
| % of 400 M budget    | **~41.9 %**      |             |

The pairing-only cost is verified by CI assertion (`cost <= 400_000_000`).

## CI Integration

The benchmark suite runs via `cargo test -p zk-soroban -- --nocapture`
(or `make bench`):

- All single-primitive asserts enforce `cost ≤ 100,000,000`.
- `groth16_verify` asserts `cost ≤ 400,000,000`.
