fn main() {
    // Correct paths for ark-bn254 v0.4.x
    use ark_bn254::G2Affine;
    use ark_ec::AffineRepr; // Use this to access the generator
    use ark_ff::PrimeField;
    use num_bigint::BigUint; // Necessary for converting field elements to BigUint

    let g2 = G2Affine::generator();

    // Arkworks G2 elements are represented as struct with c0 and c1
    let x_c0: BigUint = g2.x.c0.into_bigint().into();
    let x_c1: BigUint = g2.x.c1.into_bigint().into();
    let y_c0: BigUint = g2.y.c0.into_bigint().into();
    let y_c1: BigUint = g2.y.c1.into_bigint().into();

    println!("G2 X c0 (Real): {:x}", x_c0);
    println!("G2 X c1 (Imag): {:x}", x_c1);
    println!("G2 Y c0 (Real): {:x}", y_c0);
    println!("G2 Y c1 (Imag): {:x}", y_c1);
}
