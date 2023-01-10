use anyhow::bail;
use lazy_static::lazy_static;
use num_bigint_dig::{BigInt, BigUint, ModInverse, ToBigInt, ToBigUint};
use num_integer::Integer;
use num_traits::{Num, One, Pow, Zero};

use crate::libs::finite_field::FiniteField;

use super::signature::Signature;

lazy_static! {
    pub static ref SECP256K1GENS_X: BigUint = BigUint::from_str_radix(
        "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
        16u32
    )
    .unwrap();
    pub static ref SECP256K1GENS_Y: BigUint = BigUint::from_str_radix(
        "483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
        16u32
    )
    .unwrap();
    pub static ref SECP256K1GENS: EccPoint = EccPoint::new_secp256k1(
        (*SECP256K1GENS_X).to_bigint().unwrap().clone(),
        (*SECP256K1GENS_Y).to_bigint().unwrap().clone()
    );
    pub static ref SECP256K1GENS_ORDER: BigUint = BigUint::from_str_radix(
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
        16u32
    )
    .unwrap();
}

// secp256k1
// y^2 == x^3 + 5x + 7
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ecc {
    pub a: BigInt,
    pub b: BigInt,
    pub x: FiniteField,
    pub y: FiniteField,
    order: Option<BigUint>,
}

impl Ecc {
    fn raw_new(
        a: BigInt,
        b: BigInt,
        x: FiniteField,
        y: FiniteField,
        order: Option<BigUint>,
    ) -> Self {
        if x.prime != y.prime {
            panic!("All FiniteField should have same prime element");
        }

        let a_wrap = FiniteField::raw_new(a.clone(), x.prime.clone());
        let b_wrap = FiniteField::raw_new(b.clone(), x.prime.clone());

        let two = FiniteField::raw_new(2.to_bigint().unwrap(), x.prime.clone());
        let three = FiniteField::raw_new(3.to_bigint().unwrap(), x.prime.clone());

        let lhs = (&y).pow(two);
        let rhs = (&x).pow(three) + (&a_wrap * &x) + &b_wrap;

        if lhs != rhs {
            panic!(
                "Point ({}, {}) is not on `y^2 == x^3 + {}*x + {}` (LHS = {}, RHS == {})",
                &x, &y, &a, &b, &lhs, &rhs
            );
        }

        Ecc { a, b, x, y, order }
    }

    fn new(a: BigInt, b: BigInt, x: FiniteField, y: FiniteField) -> Self {
        Ecc::raw_new(a, b, x, y, None)
    }

    fn new_secp256k1(x: BigInt, y: BigInt) -> Self {
        let prime = 2.to_bigint().unwrap().pow(256u64)
            - 2.to_bigint().unwrap().pow(32u64)
            - 977.to_bigint().unwrap();
        Ecc::raw_new(
            BigInt::zero(),
            7.to_bigint().unwrap(),
            FiniteField::new(x, prime.clone()),
            FiniteField::new(y, prime.clone()),
            Some((*SECP256K1GENS_ORDER).clone()),
        )
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EccPoint {
    Point(Box<Ecc>),
    PointAtInfinity,
}

impl EccPoint {
    pub fn new(a: BigInt, b: BigInt, x: FiniteField, y: FiniteField) -> Self {
        EccPoint::Point(Box::new(Ecc::new(a, b, x, y)))
    }

    pub fn new_secp256k1(x: BigInt, y: BigInt) -> Self {
        EccPoint::Point(Box::new(Ecc::new_secp256k1(x, y)))
    }

    pub fn new_from_existing(&self, x: BigInt, y: BigInt) -> Option<Self> {
        match self {
            EccPoint::Point(ecc) => Some(EccPoint::Point(Box::new(Ecc::new(
                ecc.a.clone(),
                ecc.b.clone(),
                FiniteField::new(x, ecc.x.prime.clone()),
                FiniteField::new(y, ecc.x.prime.clone()),
            )))),
            EccPoint::PointAtInfinity => None,
        }
    }

    fn verify() {
        todo!()
    }

    pub fn verify_secp256k1(&self, z: &BigUint, sig: &Signature) -> anyhow::Result<()> {
        let s_inv = &sig
            .s
            .clone()
            .mod_inverse((*SECP256K1GENS_ORDER).clone())
            .unwrap()
            .to_biguint()
            .unwrap();
        let u = BigUint::modpow(
            &(z * s_inv),
            &BigUint::one(),
            &(*SECP256K1GENS_ORDER).clone(),
        );
        let v = BigUint::modpow(
            &(&sig.r * s_inv),
            &BigUint::one(),
            &(*SECP256K1GENS_ORDER).clone(),
        );
        // let point = EccPoint::new_secp256k1(px, py);
        let gen_point = EccPoint::new_secp256k1(
            (*SECP256K1GENS_X).to_bigint().unwrap().clone(),
            (*SECP256K1GENS_Y).to_bigint().unwrap().clone(),
        );
        let res = u * gen_point + v * self;

        if let EccPoint::Point(point) = res {
            if point.x.num
                == sig
                    .r
                    .to_bigint()
                    .unwrap_or_else(|| panic!("Signature.r is negative number"))
            {
                return Ok(());
            }
        }
        bail!("secp256k1 verification failed")
    }

    pub fn serialize_sec(&self) -> Vec<u8> {
        if let EccPoint::Point(point) = self {
            let mut prepend: Vec<u8> = vec![4];
            let x = point.x.num.to_bytes_be().1;
            let y = point.y.num.to_bytes_be().1;
            prepend.extend(x);
            prepend.extend(y);
            prepend
        } else {
            vec![0, 0]
        }
    }

    pub fn serialize_sec_compressed(&self) -> Vec<u8> {
        if let EccPoint::Point(point) = self {
            let mut prepend: Vec<u8>;
            if point.y.num.is_odd() {
                prepend = vec![3];
            } else {
                prepend = vec![2];
            }
            let x = point.x.num.to_bytes_be().1;
            prepend.extend(x);
            prepend
        } else {
            vec![0, 0]
        }
    }
}

overloading!((lhs : EccPoint) + (rhs : EccPoint) => EccPoint as {
    match (lhs, rhs) {
        (EccPoint::PointAtInfinity, EccPoint::PointAtInfinity) => {
            panic!("[EccPoint::add] Can't add PAI with PAI")
        }
        (other, EccPoint::PointAtInfinity) => {
            other.clone()
        }
        (EccPoint::PointAtInfinity, other)=> {
            other.clone()
        }
        (EccPoint::Point(l), EccPoint::Point(r)) => {
            if l.a != r.a && l.b != r.b {
                panic!("[EccPoint::add] add two differenct ecc (lhs: a = {}, b = {} rhs: a = {}, b = {})", l.a, l.b, r.a, r.b)
            }

            if l.x.prime != r.x.prime && l.y.prime != r.y.prime {
                panic!("[EccPoint::add] Two points has different prime (lhs's prime: ({}, {}) rhs's prime: ({}, {})", &l.x.prime, &l.y.prime, &r.x.prime, &r.y.prime)
            }

            let two = FiniteField::raw_new(2.to_bigint().unwrap(), l.x.prime.clone());
            let three = FiniteField::raw_new(3.to_bigint().unwrap(), l.x.prime.clone());
            let a_wrap = FiniteField::raw_new(l.a.clone(), l.x.prime.clone());

            match (&l.x, &l.y, &r.x, &r.y) {
                (lx, ly, rx, ry) if lx == rx && ly == ry => {
                    let s = (lx * lx * three + a_wrap) / (ly * &two);
                    let new_x = &s * &s - lx * &two;
                    let new_y = s * (lx - &new_x) - ly.clone();
                    EccPoint::Point(Box::new(Ecc::new(l.a.clone(), l.b.clone(), new_x, new_y)))
                }
                (lx, _, rx, _) if lx == rx => EccPoint::PointAtInfinity,
                (lx, ly, rx, ry) => {
                    let s = (ry - ly) / (rx - lx);
                    let new_x = &s * &s - (lx + rx);
                    let new_y = s * (lx - &new_x) - ly.clone();
                    EccPoint::Point(Box::new(Ecc::new(l.a.clone(), l.b.clone(), new_x, new_y)))
                }
            }
        }
    }
});

overloading!(^(lhs : BigUint) * (rhs : EccPoint) => EccPoint as {
    match (lhs, rhs) {
        (l, _) if l == BigUint::zero() => {
            panic!("Cannot multiple zero or minus value to EccPoint")
        }
        (l, r) if l == BigUint::one() => {
            r.clone()
        }
        (l, r) => {
            let mut coef;
            if let EccPoint::Point(ecc) = r.clone() {
                if let Some(order) = &ecc.order {
                    coef = l.clone() % order;
                } else {
                    coef = l.clone();
                }
            } else {
                coef = l.clone();
            }
            let mut current = r.clone();
            let mut res = EccPoint::PointAtInfinity;
            loop {
                match &coef {
                    c if c > &BigUint::zero() => {
                        if (c & BigUint::one()) > BigUint::zero() {
                            res = res + current.clone();
                        }
                        current = &current + &current;
                        coef >>= 1;
                    }
                    _ => { break; }
                }
            }
            res
        }
    }
});

overloading!((lhs : BigUint) * ^(rhs : EccPoint) => EccPoint as {
    match (lhs, rhs) {
        (l, _) if l == &BigUint::zero() => {
            panic!("Cannot multiple zero or minus value to EccPoint")
        }
        (l, r) if l == &BigUint::one() => {
            r.clone()
        }
        (l, r) => {
            let mut coef;
            if let EccPoint::Point(ecc) = r.clone() {
                if let Some(order) = &ecc.order {
                    coef = l.clone() % order;
                } else {
                    coef = l.clone();
                }
            } else {
                coef = l.clone();
            }
            let mut current = r.clone();
            let mut res = EccPoint::PointAtInfinity;
            loop {
                match &coef {
                    c if c > &BigUint::zero() => {
                        if (c & BigUint::one()) > BigUint::zero() {
                            res = res + current.clone();
                        }
                        current = &current + &current;
                        coef >>= 1;
                    }
                    _ => { break; }
                }
            }
            res
        }
    }
});

#[cfg(test)]
mod tests {
    use std::fmt::format;

    use num_bigint_dig::{BigInt, BigUint, ModInverse, ToBigInt, ToBigUint};
    use num_traits::{Num, One};

    use crate::libs::{finite_field::FiniteField, key::Key, signature::Signature};

    use super::{Ecc, EccPoint, SECP256K1GENS_ORDER, SECP256K1GENS_X, SECP256K1GENS_Y};

    #[test]
    fn secp256k1_test1() {
        let _e1 = EccPoint::new(
            1.to_bigint().unwrap(),
            6.to_bigint().unwrap(),
            FiniteField::new(2.to_bigint().unwrap(), (11).to_bigint().unwrap()),
            FiniteField::new((7).to_bigint().unwrap(), (11).to_bigint().unwrap()),
        );
    }

    #[test]
    #[should_panic]
    fn secp256k1_test2() {
        let _e1 = EccPoint::new(
            5.to_bigint().unwrap(),
            12.to_bigint().unwrap(),
            FiniteField::new(2.to_bigint().unwrap(), (11).to_bigint().unwrap()),
            FiniteField::new((7).to_bigint().unwrap(), (11).to_bigint().unwrap()),
        );
    }

    #[test]
    fn ecc_add1() {
        // a = (2, 7) over Z11
        // 2a = (5, 2)
        let a = EccPoint::new(
            1.to_bigint().unwrap(),
            6.to_bigint().unwrap(),
            FiniteField::new(2.to_bigint().unwrap(), 11.to_bigint().unwrap()),
            FiniteField::new(7.to_bigint().unwrap(), 11.to_bigint().unwrap()),
        );
        let a2 = a
            .new_from_existing(5.to_bigint().unwrap(), 2.to_bigint().unwrap())
            .unwrap();
        let lhs = &a + &a;
        assert_eq!(lhs, a2);
    }

    #[test]
    fn ecc_mul1() {
        // a = (2, 7) over Z11
        // 10a = (8, 8)
        let a = EccPoint::new(
            1.to_bigint().unwrap(),
            6.to_bigint().unwrap(),
            FiniteField::new(2.to_bigint().unwrap(), 11.to_bigint().unwrap()),
            FiniteField::new(7.to_bigint().unwrap(), 11.to_bigint().unwrap()),
        );
        let a2 = a
            .new_from_existing(8.to_bigint().unwrap(), 8.to_bigint().unwrap())
            .unwrap();
        let cons = 10.to_biguint().unwrap();
        let lhs = cons * a;
        assert_eq!(lhs, a2);
    }

    #[test]
    fn ecc_lt_pointatinfinity() {
        // a = (2, 7) over Z11
        // 15a = 2a
        let a = EccPoint::new(
            1.to_bigint().unwrap(),
            6.to_bigint().unwrap(),
            FiniteField::new(2.to_bigint().unwrap(), 11.to_bigint().unwrap()),
            FiniteField::new(7.to_bigint().unwrap(), 11.to_bigint().unwrap()),
        );
        let a2 = a
            .new_from_existing(5.to_bigint().unwrap(), 2.to_bigint().unwrap())
            .unwrap();
        let cons = 15.to_biguint().unwrap();
        let lhs = cons * a;
        assert_eq!(lhs, a2);
    }

    #[test]
    fn ecc_pointatinfinity() {
        let a = EccPoint::new(
            1.to_bigint().unwrap(),
            6.to_bigint().unwrap(),
            FiniteField::new(2.to_bigint().unwrap(), 11.to_bigint().unwrap()),
            FiniteField::new(7.to_bigint().unwrap(), 11.to_bigint().unwrap()),
        );
        let a2 = EccPoint::PointAtInfinity;
        let cons = 13.to_biguint().unwrap();
        let lhs = cons * a;
        assert_eq!(lhs, a2);
    }

    #[test]
    fn ecc_pointatinfinity2() {
        let a = EccPoint::new(
            0.to_bigint().unwrap(),
            7.to_bigint().unwrap(),
            FiniteField::new(15.to_bigint().unwrap(), 223.to_bigint().unwrap()),
            FiniteField::new(86.to_bigint().unwrap(), 223.to_bigint().unwrap()),
        );
        let a2 = EccPoint::PointAtInfinity;
        let cons = 7.to_biguint().unwrap();
        let lhs = cons * a;
        assert_eq!(lhs, a2);
    }

    #[test]
    fn ecc_add2() {
        // (170, 142) over Z_223
        // (170, 142) + (60, 139) == (220, 181)
        let a = EccPoint::new(
            0.to_bigint().unwrap(),
            7.to_bigint().unwrap(),
            FiniteField::new(170.to_bigint().unwrap(), 223.to_bigint().unwrap()),
            FiniteField::new(142.to_bigint().unwrap(), 223.to_bigint().unwrap()),
        );
        let a2 = a
            .new_from_existing(60.to_bigint().unwrap(), 139.to_bigint().unwrap())
            .unwrap();
        let res = a
            .new_from_existing(220.to_bigint().unwrap(), 181.to_bigint().unwrap())
            .unwrap();
        let lhs = a + a2;
        assert_eq!(lhs, res);
    }

    #[test]
    fn ecc_add3() {
        // (47, 71) over Z_223
        // (47, 71) + (17, 56) == ()
        let a = EccPoint::new(
            0.to_bigint().unwrap(),
            7.to_bigint().unwrap(),
            FiniteField::new(47.to_bigint().unwrap(), 223.to_bigint().unwrap()),
            FiniteField::new(71.to_bigint().unwrap(), 223.to_bigint().unwrap()),
        );
        let a2 = a
            .new_from_existing(17.to_bigint().unwrap(), 56.to_bigint().unwrap())
            .unwrap();
        let res = a
            .new_from_existing(215.to_bigint().unwrap(), 68.to_bigint().unwrap())
            .unwrap();
        let lhs = a + a2;
        assert_eq!(lhs, res);
    }

    #[test]
    fn ecc_add4() {
        // (47, 71) over Z_223
        // (47, 71) + (17, 56) == ()
        let a = EccPoint::new(
            0.to_bigint().unwrap(),
            7.to_bigint().unwrap(),
            FiniteField::new(143.to_bigint().unwrap(), 223.to_bigint().unwrap()),
            FiniteField::new(98.to_bigint().unwrap(), 223.to_bigint().unwrap()),
        );
        let a2 = a
            .new_from_existing(76.to_bigint().unwrap(), 66.to_bigint().unwrap())
            .unwrap();
        let res = a
            .new_from_existing(47.to_bigint().unwrap(), 71.to_bigint().unwrap())
            .unwrap();
        let lhs = a + a2;
        assert_eq!(lhs, res);
    }

    #[test]
    fn secp256k1_generator1() {
        EccPoint::new_secp256k1(
            (*SECP256K1GENS_X).to_bigint().unwrap(),
            (*SECP256K1GENS_Y).to_bigint().unwrap(),
        );
    }

    #[test]
    fn secp256k1_generator_order_test1() {
        let gen = EccPoint::new_secp256k1(
            (*SECP256K1GENS_X).to_bigint().unwrap(),
            (*SECP256K1GENS_Y).to_bigint().unwrap(),
        );
        assert_eq!(
            (*SECP256K1GENS_ORDER).clone() * &gen,
            EccPoint::PointAtInfinity
        );
    }

    #[test]
    fn secp256k1_raw_signature_validate_test1() {
        let z = BigUint::from_str_radix(
            "bc62d4b80d9e36da29c16c5d4d9f11731f36052c72401a76c23c0fb5a9b74423",
            16u32,
        )
        .unwrap();
        let r = BigUint::from_str_radix(
            "37206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c6",
            16u32,
        )
        .unwrap();
        let s = BigUint::from_str_radix(
            "8ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec",
            16u32,
        )
        .unwrap();
        let px = BigInt::from_str_radix(
            "04519fac3d910ca7e7138f7013706f619fa8f033e6ec6e09370ea38cee6a7574",
            16u32,
        )
        .unwrap();
        let py = BigInt::from_str_radix(
            "82b51eab8c27c66e26c858a079bcdf4f1ada34cec420cafc7eac1a42216fb6c4",
            16u32,
        )
        .unwrap();

        let s_inv = &s
            .mod_inverse((*SECP256K1GENS_ORDER).clone())
            .unwrap()
            .to_biguint()
            .unwrap();
        let u = BigUint::modpow(&(&z * s_inv), &BigUint::one(), &SECP256K1GENS_ORDER);
        let v = BigUint::modpow(&(&r * s_inv), &BigUint::one(), &SECP256K1GENS_ORDER);
        let point = EccPoint::new_secp256k1(px, py);
        let gen_point = EccPoint::new_secp256k1(
            (*SECP256K1GENS_X).to_bigint().unwrap(),
            (*SECP256K1GENS_Y).to_bigint().unwrap(),
        );
        if let EccPoint::Point(lhs) = u * gen_point + v * point {
            assert_eq!(lhs.x.num, r.to_bigint().unwrap());
        } else {
            panic!("POA");
        }
    }

    #[test]
    fn secp256k1_signature_validate_test1() {
        let z = BigUint::from_str_radix(
            "bc62d4b80d9e36da29c16c5d4d9f11731f36052c72401a76c23c0fb5a9b74423",
            16u32,
        )
        .unwrap();
        let r = BigUint::from_str_radix(
            "37206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c6",
            16u32,
        )
        .unwrap();
        let s = BigUint::from_str_radix(
            "8ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec",
            16u32,
        )
        .unwrap();
        let px = BigInt::from_str_radix(
            "04519fac3d910ca7e7138f7013706f619fa8f033e6ec6e09370ea38cee6a7574",
            16u32,
        )
        .unwrap();
        let py = BigInt::from_str_radix(
            "82b51eab8c27c66e26c858a079bcdf4f1ada34cec420cafc7eac1a42216fb6c4",
            16u32,
        )
        .unwrap();
        let sig = Signature::new(r, s);
        let point = EccPoint::new_secp256k1(px, py);
        point.verify_secp256k1(&z, &sig).unwrap();
    }

    #[test]
    fn sec_test() {
        let k = Key::new(5000.to_biguint().unwrap());
        let sec = k.point.serialize_sec();
        assert_eq!(sec.len(), 65);
        let lhs = hex::encode(sec);
        assert_eq!(lhs, "04ffe558e388852f0120e46af2d1b370f85854a8eb0841811ece0e3e03d282d57c315dc72890a4f10a1481c031b03b351b0dc79901ca18a00cf009dbdb157a1d10")
    }
}
