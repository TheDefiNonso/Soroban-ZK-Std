# ZK-Soroban: Fiat-Shamir Transcript Rules

This document outlines the transcript construction rules for the Fiat-Shamir heuristic within the ZK-Soroban framework, ensuring non-interactive proof security.

## 1. Transcript Initialization
The transcript must be initialized with the system's public parameters to bind the proof to the specific circuit and configuration.
- **Components:** `[DomainSeparator || CircuitID || PublicParameters]`

## 2. Interactive-to-Non-Interactive Conversion
To achieve the non-interactive property, every challenge $c$ must be derived from the hash of all preceding transcript elements.

### Rule: Sequential Commitment
1.  **Commitment Phase:** Generate a commitment $A$ (e.g., Pederson commitment).
2.  **Challenge Generation:** Compute $c = 	ext{Hash}(	ext{Transcript} || A)$.
3.  **Update Transcript:** Append $A$ and $c$ to the transcript.
    - `Transcript_{new} = Transcript_{old} || A || c`

## 3. Transcript Integrity Requirements
- **Domain Separation:** Every distinct proof step must use a unique domain separator prefix to prevent cross-protocol replay attacks.
- **Fixed-Length Encoding:** All transcript elements must be encoded using a canonical, fixed-length byte representation (e.g., Big-Endian serialization).
- **Binding Property:** The transcript must include the entire set of public inputs/outputs associated with the statement being proven.

## 4. Security Constraints
- **Collision Resistance:** The hash function used must be cryptographically collision-resistant (e.g., SHA-256 or Poseidon for ZK-friendly circuits).
- **No External State:** The challenge derivation must strictly depend on the provided transcript elements. Do not include volatile external state (e.g., timestamps) in the hash input.
