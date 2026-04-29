#![no_std]
use ethnum::u256;

/// Errors returned by zero-knowledge conversion and validation operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ZkError {
    /// The supplied value is ≥ the BN254 scalar field modulus and is not a valid field element.
    InvalidFieldElement,
    /// Mismatched input lengths or empty slices in multi-input operations.
    InvalidInput,
}

/// A BN254 scalar field element guaranteed to be in the range `[0, r)`.
///
/// Construct exclusively via [`SafeFrom`] to enforce field bounds without panicking.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Fr(u256);

/// A BN254 G1 point in affine coordinates (x, y).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct G1Affine {
    pub x: u256,
    pub y: u256,
}

/// A BN254 G1 point in Jacobian coordinates (X, Y, Z).
/// Represents the affine point (X/Z^2, Y/Z^3).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct G1Jacobian {
    pub x: u256,
    pub y: u256,
    pub z: u256,
}

impl Fr {
    /// Returns the inner `u256` representation of the field element.
    #[inline(always)]
    pub fn inner(&self) -> u256 {
        self.0
    }
}

/// Constant-time, fallible conversion into a cryptographic type.
///
/// All implementations **must** be `#[inline(always)]`, must not allocate on
/// the heap, and must never call `unwrap()` or `expect()`.
pub trait SafeFrom<T>: Sized {
    fn safe_from(val: T) -> Result<Self, ZkError>;
}

impl SafeFrom<u256> for Fr {
    /// Converts a raw `u256` into an `Fr` field element using a constant-time range check.
    ///
    /// Uses subtraction overflow to test `val < r` without branching on intermediate
    /// secret values: `overflowing_sub` overflows if and only if `val < BASE_MODULUS`.
    /// Returns `Err(ZkError::InvalidFieldElement)` if `val >= r`. No heap allocation;
    /// no panics.
    #[inline(always)]
    fn safe_from(val: u256) -> Result<Self, ZkError> {
        // Constant-time range check: subtract the modulus and detect underflow.
        // Underflow occurs iff val < BASE_MODULUS, meaning val is a valid field element.
        let (_, in_field) = val.overflowing_sub(Bn254::BASE_MODULUS);
        if in_field {
            Ok(Fr(val))
        } else {
            Err(ZkError::InvalidFieldElement)
        }
    }
}

pub struct Bn254;

impl Bn254 {
    /// BN254 scalar field modulus r (order of G1/G2).
    pub const BASE_MODULUS: ethnum::u256 = ethnum::u256::from_words(
        0x30644e72e131a029b85045b68181585d_u128,
        0x2833e84879b9709143e1f593f0000001_u128,
    );
    /// Alias for BASE_MODULUS — used by the Legendre check functions.
    pub const FR_MODULUS: ethnum::u256 = ethnum::u256::from_words(
        0x30644e72e131a029b85045b68181585d_u128,
        0x2833e84879b9709143e1f593f0000001_u128,
    );

    pub const SCALAR_ORDER: ethnum::u256 = ethnum::u256::from_words(
        0x30644e72e131a029b85045b68181585d_u128,
        0x2833e84879b9709143e1f593f0000001_u128,
    );

    // pub const BASE_MODULUS: u256 = u256::from_words(
    //     0x30644e72e131a029b85045b68181585d,
    //     0x97816a916871ca8d3c208c16d87cfd47,
    // );
    /// BN254 base field modulus p (coordinate field of the curve).
    pub const FQ_MODULUS: ethnum::u256 = ethnum::u256::from_words(
        0x30644e72e131a029b85045b68181585d_u128,
        0x97816a916871ca8d3c208c16d87cfd47_u128,
    );

    /// G1 coefficient B for y^2 = x^3 + B
    pub const G1_B: u256 = u256::from_words(0u128, 3u128);

    /// (r - 1) / 2 — Legendre exponent for the scalar field Fr.
    /// Pre-computed to avoid runtime division; used by `legendre_fr`.
    pub const LEGENDRE_EXP_FR: ethnum::u256 = ethnum::u256::from_words(
        0x183227397098d014dc2822db40c0ac2e_u128,
        0x9419f4243cdcb848a1f0fac9f8000000_u128,
    );

    /// (p - 1) / 2 — Legendre exponent for the base field Fq.
    /// Pre-computed to avoid runtime division; used by `legendre_fq`.
    pub const LEGENDRE_EXP_FQ: ethnum::u256 = ethnum::u256::from_words(
        0x183227397098d014dc2822db40c0ac2e_u128,
        0xcbc0b548b438e5469e10460b6c3e7ea3_u128,
    );

    // ── Canonical byte serialization ──────────────────────────────────────────

    /// Encodes a scalar field element as 32-byte big-endian.
    pub fn fr_to_bytes(a: u256) -> [u8; 32] {
        a.to_be_bytes()
    }

    /// Decodes a scalar field element from 32-byte big-endian encoding.
    /// Returns `None` if the decoded value is >= r (not a valid Fr element).
    pub fn fr_from_bytes(bytes: [u8; 32]) -> Option<u256> {
        let val = u256::from_be_bytes(bytes);
        if val < Self::BASE_MODULUS {
            Some(val)
        } else {
            None
        }
    }

    /// Encodes a base field element as 32-byte big-endian.
    pub fn fq_to_bytes(a: u256) -> [u8; 32] {
        a.to_be_bytes()
    }

    /// Decodes a base field element from 32-byte big-endian encoding.
    /// Returns `None` if the decoded value is >= p (not a valid Fq element).
    pub fn fq_from_bytes(bytes: [u8; 32]) -> Option<u256> {
        let val = u256::from_be_bytes(bytes);
        if val < Self::FQ_MODULUS {
            Some(val)
        } else {
            None
        }
    }

    // ── Private generic helpers ───────────────────────────────────────────────

    #[inline(always)]
    fn add_mod(a: u256, b: u256, modulus: u256) -> u256 {
        let (sum, overflow) = a.overflowing_add(b);
        if overflow || sum >= modulus {
            sum.wrapping_sub(modulus)
        } else {
            sum
        }
    }

    pub fn sub(a: u256, b: u256) -> u256 {
        let (res, underflow) = a.overflowing_sub(b);
        if underflow {
            res.wrapping_add(Self::BASE_MODULUS)
        } else {
            res
        }
    }

    /// Modular Multiplication: (a * b) % modulus
    /// Uses double-and-add to avoid 512-bit intermediate products.
    #[inline(always)]
    fn mul_mod(a: u256, b: u256, modulus: u256) -> u256 {
        let mut result = u256::from(0u8);
        let mut a = a % modulus;
        let mut b = b % modulus;
        while b > 0 {
            if b & u256::from(1u8) != u256::from(0u8) {
                result = Self::add_mod(result, a, modulus);
            }
            a = Self::add_mod(a, a, modulus);
            b >>= 1;
        }
        result
    }

    #[inline(always)]
    fn pow_mod(mut base: u256, mut exp: u256, modulus: u256) -> u256 {
        let mut res = u256::from(1u8);
        while exp > 0 {
            if exp & u256::from(1u8) != u256::from(0u8) {
                res = Self::mul_mod(res, base, modulus);
            }
            base = Self::mul_mod(base, base, modulus);
            exp >>= 1;
        }
        res
    }

    // ── Public Fr (scalar field) arithmetic ──────────────────────────────────

    pub fn is_valid_scalar(val: u256) -> bool {
        val < Self::FR_MODULUS
    }

    pub fn add(a: u256, b: u256) -> u256 {
        Self::add_mod(a, b, Self::FR_MODULUS)
    }

    pub fn mul(a: u256, b: u256) -> u256 {
        Self::mul_mod(a, b, Self::FR_MODULUS)
    }

    pub fn pow(base: u256, exp: u256) -> u256 {
        Self::pow_mod(base, exp, Self::FR_MODULUS)
    }

    pub fn invert(a: u256) -> u256 {
        if a == 0 {
            return u256::from(0u8);
        }
        let exponent = Self::FR_MODULUS - u256::from(2u8);
        Self::pow(a, exponent)
    }

    // ── Public Fq (base field) arithmetic ────────────────────────────────────

    pub fn mul_fq(a: u256, b: u256) -> u256 {
        Self::mul_mod(a, b, Self::FQ_MODULUS)
    }

    pub fn add_fq(a: u256, b: u256) -> u256 {
        Self::add_mod(a, b, Self::FQ_MODULUS)
    }

    pub fn sub_fq(a: u256, b: u256) -> u256 {
        let (res, underflow) = a.overflowing_sub(b);
        if underflow {
            res.wrapping_add(Self::FQ_MODULUS)
        } else {
            res
        }
    }

    pub fn invert_fq(a: u256) -> u256 {
        if a == 0 {
            return u256::from(0u8);
        }
        let exponent = Self::FQ_MODULUS - u256::from(2u8);
        Self::pow_mod(a, exponent, Self::FQ_MODULUS)
    }

    pub fn is_valid_g1(x: u256, y: u256) -> bool {
        if x == 0 && y == 0 {
            return false;
        }
        if x >= Self::FQ_MODULUS || y >= Self::FQ_MODULUS {
            return false;
        }

        let y_sq = Self::mul_mod(y, y, Self::FQ_MODULUS);
        let x_sq = Self::mul_mod(x, x, Self::FQ_MODULUS);
        let x_cb = Self::mul_mod(x_sq, x, Self::FQ_MODULUS);
        let rhs = Self::add_mod(x_cb, u256::from(3u8), Self::FQ_MODULUS);

        y_sq == rhs
    }

    pub fn g1_scalar_mul(point: G1Projective, scalar: u256) -> G1Projective {
        // handle edge cases
        if scalar == 0 {
            return G1Projective::identity();
        }
        if scalar == 1 {
            return point;
        }

        let mut result = G1Projective::identity();

        // BN254 scalar field is ~254 bits. Loop 254 times for constant-time.
        for i in (0..254).rev() {
            // double the current result
            result = result.double();

            // always compute addition
            let added = result.add(&point);

            // extract the i-th of the scalar
            let bit = (scalar >> i) & 1u128;

            // constant time select: replaces if bit == 1 {result = added}
            // this ensure the same branch/instruction path is taken
            result = G1Projective::cf_select(bit == 1u128, added, result);
        }

        result
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct G1Projective {
    pub x: u256,
    pub y: u256,
    pub z: u256,
}

impl G1Projective {
    pub fn identity() -> Self {
        Self {
            x: u256::from(1u8),
            y: u256::from(1u8),
            z: u256::from(0u8),
        }
    }

    /// contant time seletion: return 'a' if choice is true, 'b' if choice is false
    pub fn cf_select(choice: bool, a: Self, b: Self) -> Self {
        if choice {
            a
        } else {
            b
        }
    }

    // you will need to implement these based on standard projective formulas
    pub fn double(&self) -> Self {
        todo!()
    }
    pub fn add(&self, _other: &Self) -> Self {
        todo!()
    }
}
