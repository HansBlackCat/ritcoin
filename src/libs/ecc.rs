use lazy_static::lazy_static;
use num_bigint_dig::{BigInt, BigUint, ToBigInt, ToBigUint};
use num_traits::{Num, One, Pow, Zero};

use crate::libs::finite_field::FiniteField;

pub enum Generators {
    Secp256k1Gens(BigUint, BigUint),
    Secp256k1GroupOrder(BigUint),
}

lazy_static! {
    static ref SECP256K1GENS: Generators = Generators::Secp256k1Gens(
        BigUint::from_str_radix(
            "79BE667E F9DCBBAC 55A06295 CE870B07 029BFCDB 2DCE28D9 59F2815B 16F81798",
            16u32
        )
        .unwrap(),
        BigUint::from_str_radix(
            "483ADA77 26A3C465 5DA4FBFC 0E1108A8 FD17B448 A6855419 9C47D08F FB10D4B8",
            16u32
        )
        .unwrap()
    );
}

// secp256k1
// y^2 == x^3 + 5x + 7
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
struct Ecc {
    a: BigInt,
    b: BigInt,
    x: FiniteField,
    y: FiniteField,
}

impl Ecc {
    fn new(a: BigInt, b: BigInt, x: FiniteField, y: FiniteField) -> Self {
        if &x.prime != &y.prime {
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

        Ecc { a, b, x, y }
    }

    fn new_secp256k1(x: BigInt, y: BigInt) -> Self {
        let prime = 2.to_bigint().unwrap().pow(256u64)
            - 2.to_bigint().unwrap().pow(32u64)
            - 977.to_bigint().unwrap();
        Ecc::new(
            BigInt::zero(),
            7.to_bigint().unwrap(),
            FiniteField::new(x, prime.clone()),
            FiniteField::new(y, prime.clone()),
        )
    }

    fn check_factor(lhs: &Ecc, rhs: &Ecc) {
        if lhs.a != rhs.a && lhs.b != rhs.b {
            panic!("[Ecc::check_factor] differenct factor")
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
enum EccPoint {
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
                _ => panic!("[EccPoint::add] unhandled match"),
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
            let mut coef = l.clone();
            let mut current = r.clone();
            let mut res = EccPoint::PointAtInfinity;
            loop {
                match &coef {
                    c if c > &BigUint::zero() => {
                        eprintln!("c = {}", &c);
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
    use num_bigint_dig::{BigInt, BigUint, ToBigInt, ToBigUint};

    use crate::libs::finite_field::FiniteField;

    use super::{Ecc, EccPoint};

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
}
