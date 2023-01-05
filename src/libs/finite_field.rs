use num_traits::pow::Pow;
use std::{fmt::Display, str::FromStr};

use num_bigint_dig::{BigInt, ToBigInt};
use num_traits::One;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiniteField {
    pub num: BigInt,
    pub prime: BigInt,
}

impl Display for FiniteField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.num)
    }
}

impl FiniteField {
    pub fn new(num: BigInt, prime: BigInt) -> Self {
        if num >= prime || num < 1.to_bigint().unwrap() || prime <= 2.to_bigint().unwrap() {
            panic!("[FiniteField] prime ({prime}) must bigger than num ({num})");
        }
        FiniteField { num, prime }
    }

    pub fn new_from_i64(num: i64, prime: i64) -> Self {
        let num = num.to_bigint().unwrap();
        let prime = prime.to_bigint().unwrap();
        FiniteField::new(num, prime)
    }

    pub fn new_from_str(num: &str, prime: &str) -> Self {
        let num = BigInt::from_str(num).unwrap();
        let prime = BigInt::from_str(prime).unwrap();
        FiniteField::new(num, prime)
    }
}

macro_rules! _overloading_block {
    ($ops:ident, $fn_name:ident, $r_type:ty, $l_type:ty, $res_type:ty, $l_var:ident, $r_var:ident, $blck:block) => {
        impl std::ops::$ops<$r_type> for $l_type {
            type Output = $res_type;

            fn $fn_name(self, $r_var: $r_type) -> Self::Output {
                let $l_var = self;

                if $l_var.prime != $r_var.prime {
                    panic!(
                        "[FiniteField] lhs's prime {} != rhs's prime {}",
                        $l_var.prime, $r_var.prime
                    );
                }
                $blck
            }
        }
    };
}

macro_rules! _overloading_core {
    (+, $($t:tt)+) => {
        _overloading_block!(Add, add, $($t)+);
    };
    (-, $($t:tt)+) => {
        _overloading_block!(Sub, sub, $($t)+);
    };
    (*, $($t:tt)+) => {
        _overloading_block!(Mul, mul, $($t)+);
    };
    (/, $($t:tt)+) => {
        _overloading_block!(Div, div, $($t)+);
    };
}

macro_rules! overloading {
    (($l_var:ident : $l_type:ty) $ops:tt ($r_var:ident : $r_type:ty) => $res_type:ty as $blck:block) => {
        _overloading_core!($ops, $r_type, $l_type, $res_type, $l_var, $r_var, $blck);
        _overloading_core!($ops, $r_type, &$l_type, $res_type, $l_var, $r_var, $blck);
        _overloading_core!($ops, &$r_type, $l_type, $res_type, $l_var, $r_var, $blck);
        _overloading_core!($ops, &$r_type, &$l_type, $res_type, $l_var, $r_var, $blck);
    };
}

overloading!((lhs : FiniteField) + (rhs : FiniteField) => FiniteField as {
    FiniteField::new(
        BigInt::modpow(&(&lhs.num + &rhs.num), &BigInt::one(), &lhs.prime),
        lhs.prime.clone())
});
overloading!((lhs : FiniteField) - (rhs : FiniteField) => FiniteField as {
    FiniteField::new(
        BigInt::modpow(&(&lhs.num - &rhs.num), &BigInt::one(), &lhs.prime),
        lhs.prime.clone(),
    )
});
overloading!((lhs : FiniteField) * (rhs : FiniteField) => FiniteField as {
    FiniteField::new(
        BigInt::modpow(&(&lhs.num * &rhs.num), &BigInt::one(), &lhs.prime),
        lhs.prime.clone(),
    )
});
overloading!((lhs : FiniteField) / (rhs : FiniteField) => FiniteField as {
    FiniteField::new(
        BigInt::modpow(&(&lhs.num / &rhs.num), &BigInt::one(), &lhs.prime),
        lhs.prime.clone(),
    )
});

// impl ops::Add<FiniteField> for FiniteField {
//     type Output = FiniteField;

//     fn add(self, rhs: FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let sum = &self.num + &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&sum, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

// impl ops::Add<&FiniteField> for FiniteField {
//     type Output = FiniteField;

//     fn add(self, rhs: &FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let sum = &self.num + &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&sum, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

// impl<'a, 'b> ops::Add<&'a FiniteField> for &'b FiniteField {
//     type Output = FiniteField;

//     fn add(self, rhs: &FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let sum = &self.num + &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&sum, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

// impl ops::Sub<FiniteField> for FiniteField {
//     type Output = FiniteField;

//     fn sub(self, rhs: FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let sub = &self.num - &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&sub, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

// impl ops::Sub<&FiniteField> for FiniteField {
//     type Output = FiniteField;

//     fn sub(self, rhs: &FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let sub = &self.num - &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&sub, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

// impl<'a, 'b> ops::Sub<&'a FiniteField> for &'b FiniteField {
//     type Output = FiniteField;

//     fn sub(self, rhs: &FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let sub = &self.num - &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&sub, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

// impl ops::Mul<FiniteField> for FiniteField {
//     type Output = FiniteField;

//     fn mul(self, rhs: FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let mul = &self.num * &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&mul, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

// impl<'a, 'b> ops::Mul<&'a FiniteField> for &'b FiniteField {
//     type Output = FiniteField;

//     fn mul(self, rhs: &FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let mul = &self.num * &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&mul, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

// impl ops::Mul<BigInt> for FiniteField {
//     type Output = FiniteField;

//     fn mul(self, rhs: BigInt) -> Self::Output {
//         let tmp = FiniteField::new(rhs, self.prime.clone());
//         tmp * self
//     }
// }

// impl ops::Mul<BigInt> for &FiniteField {
//     type Output = FiniteField;

//     fn mul(self, rhs: BigInt) -> Self::Output {
//         let tmp = FiniteField::new(rhs, self.prime.clone());
//         &tmp * self
//     }
// }

// impl ops::Mul<&BigInt> for &FiniteField {
//     type Output = FiniteField;

//     fn mul(self, rhs: &BigInt) -> Self::Output {
//         let tmp = FiniteField::new(rhs.clone(), self.prime.clone());
//         &tmp * self
//     }
// }

// impl ops::Div<FiniteField> for FiniteField {
//     type Output = FiniteField;

//     fn div(self, rhs: FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let res = &self.num / &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&res, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

// impl<'a, 'b> ops::Div<&'a FiniteField> for &'b FiniteField {
//     type Output = FiniteField;

//     fn div(self, rhs: &'a FiniteField) -> Self::Output {
//         if self.prime != rhs.prime {
//             panic!(
//                 "[FiniteField] lhs's prime {} != rhs's prime {}",
//                 self.prime, rhs.prime
//             );
//         }

//         let res = &self.num / &rhs.num;

//         FiniteField::new(
//             BigInt::modpow(&res, &BigInt::one(), &self.prime),
//             self.prime.clone(),
//         )
//     }
// }

impl<'a, 'b> Pow<&'a FiniteField> for &'b FiniteField {
    type Output = FiniteField;

    fn pow(self, rhs: &'a FiniteField) -> Self::Output {
        if self.prime != rhs.prime {
            panic!(
                "[FiniteField] lhs's prime {} != rhs's prime {}",
                self.prime, rhs.prime
            );
        }

        let pow = BigInt::modpow(&self.num, &rhs.num, &self.prime);

        FiniteField::new(pow, self.prime.clone())
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::ToBigInt;

    use num_traits::Pow;

    use super::FiniteField;

    #[test]
    fn finite_field_new1() {
        let f = FiniteField::new(7.to_bigint().unwrap(), 13.to_bigint().unwrap());
        let q = FiniteField::new(7.to_bigint().unwrap(), 13.to_bigint().unwrap());
        let w = FiniteField::new(13.to_bigint().unwrap(), 23.to_bigint().unwrap());

        assert_eq!(f, q);
        assert_ne!(f, w);
    }

    #[test]
    #[should_panic]
    fn finite_field_new2() {
        let _q = FiniteField::new(13.to_bigint().unwrap(), 7.to_bigint().unwrap());
    }

    #[test]
    #[should_panic]
    fn finite_field_arith1() {
        let f = FiniteField::new(6.to_bigint().unwrap(), 13.to_bigint().unwrap());
        let g = FiniteField::new(3.to_bigint().unwrap(), 11.to_bigint().unwrap());
        let _w = &f + &g;
    }

    #[test]
    fn finite_field_arith2() {
        let f = FiniteField::new(7.to_bigint().unwrap(), 13.to_bigint().unwrap());
        let g = FiniteField::new(12.to_bigint().unwrap(), 13.to_bigint().unwrap());
        let h = FiniteField::new(6.to_bigint().unwrap(), 13.to_bigint().unwrap());
        assert_eq!(&(&f + &g), &h);
        assert_eq!(&(&f + &g), &h);
        assert_eq!(&(&h - &f), &g);
        assert_eq!(&(&h - &g), &f);
    }

    #[test]
    fn finite_field_arith3() {
        let f = FiniteField::new_from_str("479238412323191", "246240741295316874202930043963");
        let g = FiniteField::new_from_str("23902123424332", "246240741295316874202930043963");
        let h = FiniteField::new_from_str(
            "11454815681029821012417283412",
            "246240741295316874202930043963",
        );
        assert_eq!(&(&f * &g), &h);
        assert_eq!(&(&h / &g), &f);
        assert_eq!(&(&h / &f), &g);
    }

    #[test]
    fn finite_field_arith4() {
        // 124792233092312391^1322390213212344332 mod 246240741295316874202930043963
        let f = FiniteField::new_from_str("124792233092312391", "246240741295316874202930043963");
        let g = FiniteField::new_from_str("1322390213212344332", "246240741295316874202930043963");
        let h = FiniteField::new_from_str(
            "97379194738538270741642321900",
            "246240741295316874202930043963",
        );
        assert_eq!(f.pow(&g), h);
    }

    // #[test]
    // fn generator_test1() {
    //     // let mut gens = HashSet::new();
    //     let mut rng = thread_rng();
    //     let p = rng.gen_prime(2048);
    // }
}
