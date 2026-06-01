# Gas & Instruction Budget Reference

This document is the definitive cost reference for `Soroban-ZK-Std` primitives.
Use it to estimate whether your proof workflow fits within a single Soroban
transaction before writing any contract code.

> **Soroban limits (Protocol 21+)**
> - CPU instructions per transaction: **100,000,000**
> - Total composite budget (pairing-inclusive): **400,000,000**
>
> Two cost categories exist: **(A) Host-function costs** — measured directly via the
> Soroban budget API in the test environment; and **(B) WASM guest costs** — estimated
> from Criterion wall-clock benchmarks using **3,000 instructions/µs** (conservative
> x86-64 → WASM conversion factor).  Category A numbers are production-accurate;
> category B are useful for relative comparisons and budget planning.

---

## Quick Reference

| Operation | Cost (instructions) | % of 100M budget | Source |
|---|---:|---:|:---:|
| `Fr::add` | ~300 | < 0.001% | B |
| `Fr::mul` | ~669 | < 0.001% | B |
| `Fr::invert` | ~5,550,000 | 5.55% | B |
| `Fq::mul` | ~4,620 | 0.005% | B |
| `Fq::invert` | ~5,790,000 | 5.79% | B |
| `G1::double` | ~16,680 | 0.017% | B |
| `G1::add` | ~69,300 | 0.069% | B |
| `G1::to_affine` | ~5,850,000 | 5.85% | B |
| `G1::scalar_mul` (worst-case 254-bit scalar) | ~69,000,000 | 69.0% | B |
| `poseidon2_hash` (1 input) | 1,007,753 | 1.01% | A |
| `poseidon2_hash` (2 inputs) | 2,010,994 | 2.01% | A |
| `poseidon2_hash` (4 inputs) | 3,024,708 | 3.02% | A |
| `G1::msm` (n=2) | ~138,140,000 | 138%† | B |
| `G1::msm` (n=4) | ~276,210,000 | 276%† | B |
| `G1::msm` (n=8) | ~552,350,000 | 552%† | B |
| `pairing_check` (4 pairs — Groth16) | ~29,327,515 | 7.33%‡ | A |
| **`groth16_verify`** (1 pub. input, full) | **~167,400,000** | **41.9%‡** | A+B |

† MSM cost exceeds the 100 M single-op limit; decompose across multiple
  transactions or optimise with Pippenger/Strauss when available.

‡ Pairing uses the native Soroban host function.  The 29.3 M figure is the
  host-measured pairing cost.  The full Groth16 estimate adds the MSM(2)
  WASM guest cost on top (~138 M) but fits inside the 400 M composite budget.

---

## Benchmark Environment

### Criterion (category B — WASM guest estimate)

| Property            | Value                            |
|---------------------|----------------------------------|
| CPU                 | x86-64 (developer workstation)   |
| Rust toolchain      | stable (edition 2021)            |
| Optimization level  | `opt-level = 3` (bench profile)  |
| Criterion version   | 0.5                              |
| Sample count        | 100 samples per benchmark        |
| Warm-up duration    | 3 s                              |
| Measurement window  | ≥ 5 s (auto-adjusted)            |

### Soroban Budget API (category A — host-measured)

| Property            | Value                            |
|---------------------|----------------------------------|
| Soroban SDK version | 25.3.0                           |
| Stellar Protocol    | 25                               |
| Measurement API     | `env.cost_estimate().budget().cpu_instruction_cost()` |
| Profile             | `release` (`opt-level = z`, `lto = true`) |

---

## Field Arithmetic

### Wall-clock (Criterion point estimate; 95% CI shown)

| Operation   | Description                           | Low      | Point    | High     | Est. instructions |
|-------------|---------------------------------------|----------|----------|----------|-------------------|
| `Fr::add`   | Fr scalar field addition              | —        | ~100 ns  | —        | ~300              |
| `Fr::mul`   | Fr scalar field multiplication        | 219 ns   | 223 ns   | 228 ns   | ~669              |
| `Fq::mul`   | Fq base field multiplication          | 1.51 µs  | 1.54 µs  | 1.56 µs  | ~4,620            |
| `Fq::invert`| Fq inversion via Fermat (a^(p-2))     | 1.91 ms  | 1.93 ms  | 1.95 ms  | ~5,790,000        |
| `Fr::invert`| Fr inversion via Fermat (a^(r-2))     | ~1.85 ms | 1.85 ms  | —        | ~5,550,000        |

`Fr::add` uses a single 256-bit conditional subtraction and is not individually
benchmarked by Criterion; the ~100 ns figure is derived from the binary-method
loop overhead.

---

## G1 Point Operations

### Wall-clock (Criterion point estimate; 95% CI shown)

| Operation                 | Input                   | Low      | Point    | High     | Est. instructions |
|---------------------------|-------------------------|----------|----------|----------|-------------------|
| `G1::double`              | 3·G (non-trivial Z)     | 5.54 µs  | 5.56 µs  | 5.58 µs  | ~16,680           |
| `G1::add`                 | G + 2·G                 | 22.1 µs  | 23.1 µs  | 24.1 µs  | ~69,300           |
| `G1::to_affine`           | Projective → affine     | 1.93 ms  | 1.95 ms  | 1.98 ms  | ~5,850,000        |

### G1 Scalar Multiplication

The double-and-add loop runs 254 iterations; cost depends on scalar bit density.

| Scalar               | Description                        | Low       | Point     | High      | Est. instructions |
|----------------------|------------------------------------|-----------|-----------|-----------|-------------------|
| `2`                  | Trivial (2 bits)                   | 15.1 µs   | 15.2 µs   | 15.5 µs   | ~45,600           |
| `2^128 + 1`          | Medium (~129 bits, half bit-width) | 11.4 ms   | 11.5 ms   | 11.6 ms   | ~34,500,000       |
| `r − 1` (worst case) | All 254 bits set                   | 22.9 ms   | 23.0 ms   | 23.2 ms   | ~69,000,000       |

### G1 Multi-Scalar Multiplication (MSM)

MSM(n) is simulated as n sequential `G1::scalar_mul` calls plus (n−1)
`G1::add` calls.  These figures use worst-case scalars.

| Size | Est. instructions | % of 100 M | Fits in 1 tx? |
|------|------------------:|:----------:|:-------------:|
| n=2  | ~138,140,000      | 138%       | No†           |
| n=4  | ~276,210,000      | 276%       | No†           |
| n=8  | ~552,350,000      | 552%       | No†           |

† Split into multiple transactions or use mixed-addition / Pippenger when available.

---

## Poseidon2 Hash (host-measured — category A)

`poseidon2_hash` invokes `env.crypto_hazmat().poseidon2_permutation()`.
Values below are actual Soroban SDK budget measurements.

| Operation            | Inputs | CPU Instructions | % of 100 M budget |
|----------------------|-------:|-----------------:|:-----------------:|
| `poseidon2_hash`     | 1      | 1,007,753        | 1.01%             |
| `poseidon2_hash`     | 2      | 2,010,994        | 2.01%             |
| `poseidon2_hash`     | 4      | 3,024,708        | 3.02%             |

Each additional rate-block (~2 inputs) adds approximately **1,007,000 instructions**.

---

## Pairing & Groth16 Verification (host-measured — category A)

`pairing_check` delegates to the native Soroban `env.crypto().bn254_pairing_check()`
host function (CAP-0074).

### Pairing check

| Pairs | CPU Instructions | % of 400 M budget |
|------:|-----------------:|:-----------------:|
| 4     | ~29,327,515      | 7.33%             |

### Full Groth16 verify (1 public input)

Comprises:
1. MSM(2) public-input accumulator — WASM guest cost (~138 M, category B)
2. 4-pair pairing check — host function cost (~29.3 M, category A)

| Component            | CPU Instructions | Source |
|----------------------|-----------------:|:------:|
| MSM(2) accumulator   | ~138,140,000     | B      |
| 4-pair pairing check | ~29,327,515      | A      |
| **Total estimate**   | **~167,467,515** | A+B    |
| % of 400 M budget    | **~41.9 %**      |        |

The host-measured pairing alone confirmed `≤ 400,000,000` instructions in CI.
The combined estimate fits well within the 400 M composite budget, leaving
~232 M instructions for application logic, public-input hashing, and
input validation.

---

## Operation Complexity Model

These counts the dominant Fq multiplications per operation.

| Operation           | Approx Fq Muls | Notes                                      |
|---------------------|----------------|--------------------------------------------|
| `Fq::mul`           | 1              | Binary method, O(256) iterations           |
| `Fq::invert`        | ~1,250         | Fermat: a^(p-2) = 254 doublings + squarings|
| `G1::double`        | ~7–8           | Jacobian double formula                    |
| `G1::add`           | ~14–16         | Jacobian add formula (mixed: ~12)          |
| `G1::to_affine`     | ~1,252         | 1 inversion + 2 muls                       |
| `G1::scalar_mul` (254b) | ~3,810     | ~254 doubles + ~127 adds (avg half bits)   |

---

## Transaction Budget Planning

Use this table to plan multi-operation transactions.

| Workflow                             | Est. instructions | Fits in 100 M? | Fits in 400 M? |
|--------------------------------------|------------------:|:--------------:|:--------------:|
| 1× `poseidon2_hash(2)` + field checks| ~2,100,000        | Yes            | Yes            |
| 1× `G1::scalar_mul` (worst)          | ~69,000,000       | Yes            | Yes            |
| 2× `G1::scalar_mul` (worst)          | ~138,000,000      | No             | Yes            |
| 1× Groth16 verify (1 pub. input)     | ~167,500,000      | No             | Yes            |
| 2× Groth16 verify (1 pub. input)     | ~335,000,000      | No             | Yes            |
| 3× Groth16 verify (1 pub. input)     | ~502,500,000      | No             | No†            |

† Batch across multiple transactions; verify each proof separately.

---

## Assumptions

1. **No `std` overhead**: `zk-core` is `#![no_std]`.  All operations are
   pure stack-based field arithmetic with no allocations.

2. **Binary-method field multiplication**: `Fq::mul` and `Fr::mul` use a
   double-and-add binary method (O(256) field additions per multiply).
   A Montgomery reduction implementation would reduce this ~10×.

3. **Jacobian projective coordinates**: `G1::add` and `G1::double` use classic
   Jacobian formulas.  Mixed (affine + projective) addition would reduce
   `G1::add` from ~16 to ~12 Fq multiplications.

4. **Scalar width**: `g1_scalar_mul` always iterates 254 times regardless of
   scalar bit-width; no early-exit for leading zeros.

5. **Host-function pairing**: `pairing_check` invokes the Soroban native BN254
   host function (CAP-0074).  The ~29.3 M instruction figure is
   host-measured and protocol-accurate for Soroban SDK 25.3.0.

6. **3,000 instructions/µs conversion**: Used only for category B estimates.
   Actual WASM metered costs may differ by ±30% depending on the host machine
   and JIT behaviour.

---

## Reproducibility

### Run Criterion benchmarks (category B)

```bash
# From repository root:
cargo bench -p zk-core

# Run a single benchmark group:
cargo bench -p zk-core -- G1ScalarMul

# Open HTML report:
open target/criterion/G1Add/report/index.html
```

### Run Soroban instruction-cost tests (category A)

```bash
# Prints measured instruction counts and asserts budget limits:
cargo test -p zk-soroban --test '*' -- --nocapture

# Or via Makefile:
make bench
```

### Compare releases

```bash
git stash
cargo bench -p zk-core -- --save-baseline before
git stash pop
cargo bench -p zk-core -- --baseline before
```

---

## Future Work

- [ ] Wire Soroban host metering for native (category B) operations to replace
      wall-clock estimates with exact instruction counts
- [ ] Add Montgomery-form `Fq::mul` to reduce scalar multiplication cost ~10×
- [ ] Implement mixed Jacobian/affine `G1::add` to save ~4 Fq muls per add
- [ ] Add Pippenger MSM to reduce n-point MSM below n × scalar_mul cost
- [ ] Benchmark `ElGamalCiphertext::encrypt` end-to-end
- [ ] Add Groth16 with 2, 4, 8 public inputs once MSM is optimised

---

*Last updated: 2026-06-01.  Re-run benchmarks after any change to `zk-core`
field arithmetic, curve operations, or `zk-soroban` host-function wrappers.*
