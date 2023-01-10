use num_bigint_dig::{BigInt, BigUint, ModInverse, RandBigInt, ToBigInt, ToBigUint};
use num_traits::{One, Zero};

use super::{
    ecc::{EccPoint, SECP256K1GENS, SECP256K1GENS_ORDER},
    signature::Signature,
};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    pub secret: BigUint,
    pub point: EccPoint,
}

impl Key {
    pub fn new(secret: BigUint) -> Self {
        let point = &secret * &(*SECP256K1GENS);
        Key { secret, point }
    }

    fn rfc6979(&self, z: BigUint) -> BigUint {
        todo!()
    }

    pub fn sign(&self, z: BigUint) -> Signature {
        let mut rng = rand::thread_rng();
        let k = rng.gen_biguint_range(&BigUint::zero(), &SECP256K1GENS_ORDER);
        if let EccPoint::Point(point) = k.clone() * &(*SECP256K1GENS) {
            let r = point.x.num.clone().to_biguint().unwrap();
            let k_inv = k
                .mod_inverse(&(*SECP256K1GENS_ORDER))
                .unwrap()
                .to_biguint()
                .unwrap();
            let mut s = BigUint::modpow(
                &((z + &r * &self.secret) * k_inv),
                &BigUint::one(),
                &SECP256K1GENS_ORDER,
            );
            // Transaction's Malleability
            if s > &(*SECP256K1GENS_ORDER) / 2.to_biguint().unwrap() {
                s = &(*SECP256K1GENS_ORDER) - s;
            }
            return Signature::new(r, s);
        }
        panic!("Generator is POI");
    }
}

#[cfg(test)]
mod tests {}
