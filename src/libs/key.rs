use num_bigint_dig::{BigUint, RandBigInt};
use num_traits::Zero;

use super::{
    ecc::{EccPoint, SECP256K1GENS, SECP256K1GENS_ORDER},
    signature::Signature,
};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
struct Key {
    pub secret: BigUint,
    pub point: EccPoint,
}

impl Key {
    pub fn sign(&self, z: BigUint) -> Signature {
        let mut rng = rand::thread_rng();
        let k = rng.gen_biguint_range(&BigUint::zero(), &(*SECP256K1GENS_ORDER));
        let r;
        if let EccPoint::Point(point) = k * &(*SECP256K1GENS) {
            r = point.x.num.clone();
        } else {
            panic!("Generator is POI");
        }
        todo!()
    }
}
