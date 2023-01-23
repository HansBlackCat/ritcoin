use num_bigint_dig::{BigUint, ModInverse, RandBigInt, ToBigUint};
use num_traits::{One, Zero};

use crate::libs::ecc::Ecc;

use super::{
    ecc::{EccPoint, SECP256K1GENS, SECP256K1GENS_ORDER},
    network::BitcoinNetwork,
    signature::{base58_encode_with_checksum, Signature},
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

    fn generate_wif_raw(
        &self,
        compressed: bool,
        network: BitcoinNetwork,
    ) -> Result<String, std::string::FromUtf8Error> {
        let prefix = match network {
            BitcoinNetwork::MainNet => 0x80_u8,
            BitcoinNetwork::TestNet => 0xef_u8,
        };
        let mut res: Vec<u8> = Vec::new();
        res.push(prefix);
        eprintln!("[generate_wif_raw] prefix: {:?}", hex::encode(&res));
        res.extend(&self.secret.to_bytes_be());
        if compressed {
            res.extend(b"01");
        }
        eprintln!("[generate_wif_raw] res: {:?}", hex::encode(&res));
        base58_encode_with_checksum(&res)
    }

    pub fn generate_wif_compressed(
        &self,
        network: BitcoinNetwork,
    ) -> Result<String, std::string::FromUtf8Error> {
        self.generate_wif_raw(true, network)
    }

    pub fn generate_wif(
        &self,
        network: BitcoinNetwork,
    ) -> Result<String, std::string::FromUtf8Error> {
        self.generate_wif_raw(false, network)
    }
}

#[cfg(test)]
mod tests {
    use num_bigint_dig::BigUint;
    use num_traits::FromPrimitive;

    use crate::libs::network::BitcoinNetwork;

    use super::Key;

    #[test]
    fn secret_key_to_address() {
        let k = Key::new(BigUint::from_u128(5002_u128).unwrap());
        let t = k
            .point
            .gernerate_address_from_sec(BitcoinNetwork::TestNet)
            .unwrap();
        eprintln!("{:?}", t);
        panic!("")
    }

    #[test]
    fn wif_test() {
        let key = Key::new(BigUint::from_i32(5003).unwrap());
        let res = key.generate_wif_compressed(BitcoinNetwork::TestNet);
        eprintln!("{}", res.unwrap());
        panic!("")
    }

    #[test]
    fn wif_test2() {
        let key = Key::new(
            BigUint::parse_bytes(
                b"0C28FCA386C7A227600B2FE50B7CAE11EC86D3BF1FBE471BE89827E19D72AA1D",
                16,
            )
            .unwrap(),
        );
        eprintln!("{}", key.generate_wif(BitcoinNetwork::MainNet).unwrap());
        panic!("pan")
    }
}
