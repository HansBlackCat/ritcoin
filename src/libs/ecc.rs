// use anyhow::{Context, Ok};
// use num_bigint_dig::{BigInt, BigUint, ToBigInt};
// use num_traits::{One, Pow, Zero};

// use crate::libs::finite_field::FiniteField;

// // secp256k1
// // y^2 == x^3 + 5x + 7
// #[non_exhaustive]
// #[derive(Debug, Clone, PartialEq, Eq)]
// struct Ecc {
//     a: FiniteField,
//     b: FiniteField,
//     x: FiniteField,
//     y: FiniteField,
// }

// impl Ecc {
//     fn new(a: FiniteField, b: FiniteField, x: FiniteField, y: FiniteField) -> Self {
//         if &a.prime != &b.prime || &b.prime != &x.prime || &x.prime != &y.prime {
//             panic!("All FiniteField should have same prime element");
//         }

//         if &y.num.pow(2_u32) != &x.num.pow(3_u32) + (&a * &x) + &b {
//             panic!(
//                 "Point ({}, {}) is not on `y^2 == x^3 + {}*x + {}` (LHS = {}, RHS == {})",
//                 &x,
//                 &y,
//                 &a,
//                 &b,
//                 &(y.pow(2_u32)),
//                 &(x.pow(3_u32) + (&a * &x) + &b)
//             );
//         }

//         Ecc { a, b, x, y }
//     }

//     fn new_secp256k1(x: FiniteField, y: FiniteField) -> Self {
//         Ecc::new(
//             FiniteField::new(BigInt::one(), x.prime),
//             FiniteField::new(7.to_bigint().unwrap(), x.prime),
//             x,
//             y,
//         )
//     }

//     fn check_factor(lhs: &Ecc, rhs: &Ecc) {
//         if lhs.a != rhs.a && lhs.b != rhs.b {
//             panic!("[Ecc::check_factor] differenct factor")
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// enum EccPoint {
//     Point(Box<Ecc>),
//     PointAtInfinity,
// }

// impl EccPoint {
//     pub fn new(a: FiniteField, b: FiniteField, x: FiniteField, y: FiniteField) -> Self {
//         EccPoint::Point(Box::new(Ecc::new(a, b, x, y)))
//     }

//     pub fn new_secp256k1(x: FiniteField, y: FiniteField) -> Self {
//         EccPoint::Point(Box::new(Ecc::new_secp256k1(x, y)))
//     }

//     pub fn add(lhs: &EccPoint, rhs: &EccPoint) -> EccPoint {
//         match (lhs, rhs) {
//             (EccPoint::PointAtInfinity, EccPoint::PointAtInfinity) => {
//                 panic!("[EccPoint::add] Can't add PAI with PAI")
//             }
//             (EccPoint::PointAtInfinity, other) | (other, EccPoint::PointAtInfinity) => {
//                 other.clone()
//             }
//             (EccPoint::Point(l), EccPoint::Point(r)) => {
//                 if l.a != r.a && l.b != r.b {
//                     panic!("[EccPoint::add] add two differenct ecc (lhs: a = {}, b = {} rhs: a = {}, b = {})", l.a, l.b, r.a, r.b)
//                 }

//                 match (&l.x, &l.y, &r.x, &r.y) {
//                     (lx, ly, rx, ry) if lx == rx && ly == ry => {
//                         let s = (lx * lx * 3.to_bigint().unwrap() + &l.a)
//                             / (ly * 2.to_bigint().unwrap());
//                         let new_x = &s * &s - lx * 2.to_bigint().unwrap();
//                         let new_y = s * (lx - &new_x) - ly.clone();
//                         EccPoint::Point(Box::new(Ecc::new(l.a.clone(), l.b.clone(), new_x, new_y)))
//                     }
//                     (lx, _, rx, _) if lx == rx => EccPoint::PointAtInfinity,
//                     (lx, ly, rx, ry) => {
//                         let s = (ry - ly) / (rx - lx);
//                         let new_x = &s * &s - (lx + rx);
//                         let new_y = s * (lx - &new_x) - ly.clone();
//                         EccPoint::Point(Box::new(Ecc::new(l.a.clone(), l.b.clone(), new_x, new_y)))
//                     }
//                     _ => panic!("[EccPoint::add] unhandled match"),
//                 }
//             }
//         }
//     }

//     pub fn mul(factor: BigUint, point: &EccPoint) -> EccPoint {
//         match (&factor, point) {
//             _ if factor <= BigUint::zero() => {
//                 panic!("Cannot multiple zero or minus value to EccPoint")
//             }
//             _ => {
//                 let mut res = point.clone();
//                 for _ in num_iter::range_inclusive(BigUint::from(0u64), factor) {
//                     res = EccPoint::add(&res, point);
//                 }
//                 res
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use num_bigint_dig::{BigInt, BigUint, ToBigInt};

//     use super::{Ecc, EccPoint};

//     #[test]
//     fn secp256k1_test1() {
//         let _e1 = Ecc::new(
//             5.to_bigint().unwrap(),
//             7.to_bigint().unwrap(),
//             (-1).to_bigint().unwrap(),
//             (-1).to_bigint().unwrap(),
//         );
//     }

//     #[test]
//     #[should_panic]
//     fn secp256k1_test2() {
//         let _e1 = Ecc::new(
//             5.to_bigint().unwrap(),
//             12.to_bigint().unwrap(),
//             (-3).to_bigint().unwrap(),
//             (-1).to_bigint().unwrap(),
//         );
//     }

//     #[test]
//     fn ecc_add1() {
//         // (a, b) = (5, 7)
//         // (2, 5) + (-1, -1) == (3, -7)
//         let a = EccPoint::new(
//             5.to_bigint().unwrap(),
//             7.to_bigint().unwrap(),
//             2.to_bigint().unwrap(),
//             5.to_bigint().unwrap(),
//         );
//         let b = EccPoint::new(
//             5.to_bigint().unwrap(),
//             7.to_bigint().unwrap(),
//             (-1).to_bigint().unwrap(),
//             (-1).to_bigint().unwrap(),
//         );
//         let c = EccPoint::new(
//             5.to_bigint().unwrap(),
//             7.to_bigint().unwrap(),
//             3.to_bigint().unwrap(),
//             (-7).to_bigint().unwrap(),
//         );
//         let lhs = EccPoint::add(&a, &b);
//         let _lhs_same = EccPoint::add(&a, &b);
//         assert_eq!(lhs, c);
//     }

//     #[test]
//     fn ecc_mul1() {
//         let a = EccPoint::new(
//             1.to_bigint().unwrap(),
//             6.to_bigint().unwrap(),
//             2.to_bigint().unwrap(),
//             7.to_bigint().unwrap(),
//         );
//         let res1 = EccPoint::add(&a, &a);
//         let res2 = EccPoint::add(&res1, &a);
//         let lhs = EccPoint::mul(BigUint::from(1u64), &a);
//         assert_eq!(lhs, res1);
//         // assert_eq!(lhs, res2);
//     }
// }
