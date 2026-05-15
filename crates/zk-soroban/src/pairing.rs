use ethnum::u256;
use soroban_sdk::crypto::bn254::{Bn254G1Affine as SdkG1Affine, Bn254G2Affine as SdkG2Affine};
use soroban_sdk::BytesN;
use soroban_sdk::Env;
use soroban_sdk::Vec;
use zk_core::{G1Affine, ZkError};

/// A BN254 G2 point in affine coordinates (X, Y).
/// Coordinates are elements of the degree-2 extension field Fq²,
/// represented as `a + b*u`, where `0` is the real part and `1` is the imaginary part.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct G2Affine {
    pub x: (u256, u256),
    pub y: (u256, u256),
}

impl G2Affine {
    /// Serializes the G2 point into a 128-byte array according to CAP-0074.
    ///
    /// ## Byte Layout
    /// The 128 bytes are structured as:
    /// - Bytes 0..32:   `x.0` (X real)
    /// - Bytes 32..64:  `x.1` (X imaginary)
    /// - Bytes 64..96:  `y.0` (Y real)
    /// - Bytes 96..128: `y.1` (Y imaginary)
    ///
    /// All 32-byte chunks are encoded in Big-Endian format.
    pub fn to_bytes(&self) -> [u8; 128] {
        let mut bytes = [0u8; 128];
        // Sequence: X_c0, X_c1, Y_c0, Y_c1 (Big Endian)
        bytes[0..32].copy_from_slice(&self.x.0.to_be_bytes()); // X c0
        bytes[32..64].copy_from_slice(&self.x.1.to_be_bytes()); // X c1
        bytes[64..96].copy_from_slice(&self.y.0.to_be_bytes()); // Y c0
        bytes[96..128].copy_from_slice(&self.y.1.to_be_bytes()); // Y c1
        bytes
    }
}

/// Evaluates the BN254 pairing check e(A1, B1) * ... * e(An, Bn) == 1.
pub fn pairing_check(env: &Env, pairs: &[(G1Affine, G2Affine)]) -> Result<bool, ZkError> {
    if pairs.is_empty() {
        return Err(ZkError::InvalidInput);
    }

    let mut vp1: Vec<SdkG1Affine> = Vec::new(env);
    let mut vp2: Vec<SdkG2Affine> = Vec::new(env);

    for (g1, g2) in pairs {
        let mut g1_bytes = [0u8; 64];
        g1_bytes[0..32].copy_from_slice(&g1.x.to_be_bytes());
        g1_bytes[32..64].copy_from_slice(&g1.y.to_be_bytes());

        let g2_bytes = g2.to_bytes();

        let sdk_g1 = SdkG1Affine::from_bytes(BytesN::from_array(env, &g1_bytes));
        let sdk_g2 = SdkG2Affine::from_bytes(BytesN::from_array(env, &g2_bytes));

        vp1.push_back(sdk_g1);
        vp2.push_back(sdk_g2);
    }

    Ok(env.crypto().bn254().pairing_check(vp1, vp2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethnum::u256;
    use soroban_sdk::Env;

    #[test]
    fn test_pairing_check_rejects_empty_input() {
        let env = Env::default();
        assert_eq!(pairing_check(&env, &[]), Err(ZkError::InvalidInput));
    }

    #[test]
    fn test_bn254_pairing_identities() {
        let env = Env::default();

        let g1 = G1Affine {
            x: u256::from(1u8),
            y: u256::from(2u8),
        };

        let neg_g1 = G1Affine {
            x: u256::from(1u8),
            y: u256::from_str_radix(
                "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd45",
                16,
            )
            .unwrap(),
        };

        // Standard BN254 G2 Generator Constants
        // P = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47
        let g2 = G2Affine {
            x: (
                // X c0 (Real)
                u256::from_str_radix(
                    "10822403556616783d294cae447f68c351084c519bc131644754784460d3d548",
                    16,
                )
                .unwrap(),
                // X c1 (Imaginary)
                u256::from_str_radix(
                    "012c40590818290663486c8f967a1262d47155ec1608677c77d0a64983050961",
                    16,
                )
                .unwrap(),
            ),
            y: (
                // Y c0 (Real)
                u256::from_str_radix(
                    "0689357dbd07bdc858f01f28fd87f6b6e11802996d9ed800f1351194380126d4",
                    16,
                )
                .unwrap(),
                // Y c1 (Imaginary)
                u256::from_str_radix(
                    "24f0c4314c4083a290e2124576307135e6179426f497401c37b60514f7b603d3",
                    16,
                )
                .unwrap(),
            ),
        };

        let is_valid_pair = pairing_check(&env, &[(g1, g2), (neg_g1, g2)]).unwrap();
        assert!(is_valid_pair, "e(G1, G2) * e(-G1, G2) == 1");
    }
}
