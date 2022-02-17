use elliptic_curve::bigint::Encoding;
use elliptic_curve::{Curve, ScalarCore};
use k256::{AffinePoint, Scalar, Secp256k1};
use rug::{integer::Order, Integer};

pub fn to_scalar(x: &Integer) -> Scalar {
    let order: Integer = Integer::from_str_radix(
        "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
        16,
    )
    .unwrap();
    let mut digits = (x % order).to_digits::<u8>(Order::Lsf);
    digits.resize(<Secp256k1 as Curve>::UInt::BYTE_SIZE, 0u8);
    Scalar::from(ScalarCore::<Secp256k1>::from_le_slice(&digits).unwrap())
}

pub const G: AffinePoint = AffinePoint::GENERATOR;
pub const H: AffinePoint = AffinePoint::GENERATOR;
