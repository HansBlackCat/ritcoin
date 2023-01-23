use std::{string::FromUtf8Error, vec};

use num_bigint_dig::{BigInt, BigUint, Sign};
use num_traits::{FromPrimitive, ToPrimitive};
use sha2::{Digest, Sha256};

use crate::unwrap_or_none;

const BASE58_ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub r: BigUint,
    pub s: BigUint,
}

impl Signature {
    pub fn new(r: BigUint, s: BigUint) -> Self {
        Signature { r, s }
    }

    pub fn der(&self) -> Vec<u8> {
        let der_prefix = vec![0x30_u8];
        let mut r = trim_null_start(self.r.to_bytes_be()).unwrap();
        if &r[0] & 0x80 > 0 {
            r.splice(0..0, [0x00_u8]);
        }
        r.splice(0..0, [0x02_u8, r.len().to_u8().unwrap()]);
        eprintln!("{:?}", r);

        let mut s = self.s.to_bytes_be();
        if &s[0] & 0x80 > 0 {
            s.splice(0..0, [0x00_u8]);
        }
        s.splice(0..0, [0x02_u8, s.len().to_u8().unwrap()]);

        r.extend(&s);
        eprintln!("{:?}", r);
        let tot_len = r.len().to_u8().unwrap();
        r.splice(0..0, [tot_len]);
        r.splice(0..0, der_prefix);
        r
    }
}

pub fn trim_null_start(from: Vec<u8>) -> Option<Vec<u8>> {
    let len = from.len();
    let mut j: i64 = -1;
    for i in 0..len {
        if from[i] != 0x00 {
            j = unwrap_or_none!(i.to_i64());
            break;
        }
    }

    if j == -1 {
        panic!("[trim_null] all binaries given are null(0x00)");
    }
    let raw = &from[unwrap_or_none!(j.to_usize())..len];
    Some(raw.to_vec())
}

// TODO to iterator (Algorithm)
pub fn base58_encode(s: &[u8]) -> Result<String, FromUtf8Error> {
    let mut null_count = 0;
    for elem in s {
        if (0x00_u8).eq(elem) {
            null_count += 1;
        } else {
            break;
        }
    }

    let mut number = BigInt::from_bytes_be(Sign::Plus, s.as_ref());
    let mut result: Vec<u8> = Vec::new();
    while &number > &BigInt::from_i32(0).unwrap() {
        let divrem = num_integer::div_rem(number, BigInt::from_i32(58).unwrap());
        number = divrem.0;
        let idx = BASE58_ALPHABET[divrem.1.to_usize().unwrap()];
        result.push(idx);
    }

    for _ in 0..null_count {
        result.push(b'1');
    }
    result.reverse();
    String::from_utf8(result)
}

// TODO should shasum256 twice
// https://en.bitcoin.it/wiki/Wallet_import_format
pub fn base58_encode_with_checksum(s: &[u8]) -> Result<String, FromUtf8Error> {
    eprintln!("[base58_checksum] given: {:?}", hex::encode(s));
    let _checksum: [u8; 32] = Sha256::digest(s)
        .as_slice()
        .try_into()
        .expect("wrong slice length");
    eprintln!(
        "[base58_checksum] checksum res: {:?}",
        hex::encode(_checksum.to_vec())
    );
    let checksum: &[u8; 4] = &_checksum[0..4]
        .try_into()
        .expect("checksum must be 4 bytes");
    let mut result: Vec<u8> = Vec::new();
    result.extend(s);
    result.extend(checksum);
    base58_encode(&result)
}

#[cfg(test)]
mod tests {
    use crate::libs::signature::base58_encode_with_checksum;

    use super::base58_encode;
    use super::Signature;
    use digest::Digest;
    use num_bigint_dig::BigUint;
    use num_traits::{Num, ToPrimitive};
    use sha2::Sha256;

    #[test]
    fn trim_test() {
        let test_vec: Vec<u8> = vec![0x00, 0x00, 0x32, 0x01, 0xA1];
        let len = test_vec.len();
        let mut j: i32 = -1;
        for i in 0..len {
            if test_vec[i] != 0x00 {
                j = i.to_i32().unwrap();
                break;
            }
        }

        if j == -1 {
            panic!("all binaries are null")
        }
        let trimmed = &test_vec[j.to_usize().unwrap()..len];
        assert_eq!(trimmed.to_vec(), vec![0x32, 0x01, 0xA1])
    }

    #[test]
    fn der_test() {
        let sig = Signature::new(
            BigUint::from_str_radix(
                "37206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c6",
                16,
            )
            .unwrap(),
            BigUint::from_str_radix(
                "8ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec",
                16,
            )
            .unwrap(),
        );
        eprintln!("{}", hex::encode(&sig.der()));
        assert_eq!(
            format!("{}", hex::encode(&sig.der())),
            "3045022037206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c6022100\
        8ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec"
        );
    }

    #[test]
    fn base58_test() {
        let a = b"7c076ff316692a3d7eb3c3bb0f8b1488cf72e1afcd929e29307032997a838a3d";
        let b = b"eff69ef2b1bd93a66ed5219add4fb51e11a840f404876325a1e8ffe0529a2c";
        let c = b"c7207fee197d27c618aea621406f6bf5ef6fca38681d82b2f06fddbdce6feab6";
        let hw = b"Hello World!";
        let a_dec = hex::decode(&a).unwrap();
        eprintln!("{:?}", base58_encode(&a_dec).unwrap());
        assert_eq!(
            base58_encode(&hex::decode(&a).unwrap()).unwrap(),
            "9MA8fRQrT4u8Zj8ZRd6MAiiyaxb2Y1CMpvVkHQu5hVM6"
        );
        assert_eq!(
            base58_encode(&hex::decode(&b).unwrap()).unwrap(),
            "4fE3H2E6XMp4SsxtwinF7w9a34ooUrwWe4WsW1458Pd"
        );
        assert_eq!(
            base58_encode(&hex::decode(&c).unwrap()).unwrap(),
            "EQJsjkd6JaGwxrjEhfeqPenqHwrBmPQZjJGNSCHBkcF7"
        );
        eprintln!("{:?}", base58_encode(hw).unwrap());
        assert_eq!(base58_encode(hw).unwrap(), "2NEpo7TZRRrLZSi2U");
    }

    #[test]
    fn sha_test() {
        let s = Sha256::digest(b"Test data").to_vec();
        assert!(s.len() == 32);
        assert_eq!(
            hex::encode(&s),
            "e27c8214be8b7cf5bccc7c08247e3cb0c1514a48ee1f63197fe4ef3ef51d7e6f"
        );
    }

    #[test]
    fn base58_checksum_test() {
        let a = b"Test data";
        eprintln!("{:?}", base58_encode(a).unwrap());
        eprintln!("{:?}", base58_encode_with_checksum(a).unwrap());
        panic!("s");
    }
}
