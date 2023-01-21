use std::vec;

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

pub fn base58_encode<T: AsRef<[u8]>>(s: T) -> Vec<u8> {
    let mut null_count = 0;
    for elem in s.as_ref() {
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
    result
}

pub fn base58_encode_with_checksum<T: AsRef<[u8]>>(s: T) -> Vec<u8> {
    let t = Sha256::digest(s.as_ref());
    vec![0x00_u8]
}

#[cfg(test)]
mod tests {
    use num_bigint_dig::BigUint;
    use num_traits::{Num, ToPrimitive};

    use super::base58_encode;
    use super::Signature;

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
        eprintln!("{:?}", String::from_utf8(base58_encode(a_dec)));
        assert_eq!(
            String::from_utf8(base58_encode(hex::decode(&a).unwrap())).unwrap(),
            "9MA8fRQrT4u8Zj8ZRd6MAiiyaxb2Y1CMpvVkHQu5hVM6"
        );
        assert_eq!(
            String::from_utf8(base58_encode(hex::decode(&b).unwrap())).unwrap(),
            "4fE3H2E6XMp4SsxtwinF7w9a34ooUrwWe4WsW1458Pd"
        );
        assert_eq!(
            String::from_utf8(base58_encode(hex::decode(&c).unwrap())).unwrap(),
            "EQJsjkd6JaGwxrjEhfeqPenqHwrBmPQZjJGNSCHBkcF7"
        );
        eprintln!("{:?}", String::from_utf8(base58_encode(hw)));
        assert_eq!(
            String::from_utf8(base58_encode(hw)).unwrap(),
            "2NEpo7TZRRrLZSi2U"
        );
    }
}
