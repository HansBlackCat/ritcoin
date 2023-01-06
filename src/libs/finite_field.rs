use num_traits::pow::Pow;
use std::{fmt::Display, str::FromStr};

use num_bigint_dig::{BigInt, ModInverse, ToBigInt};
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
    pub fn raw_new(num: BigInt, prime: BigInt) -> Self {
        if prime <= 2.to_bigint().unwrap() {
            // panic!("[FiniteField] prime ({prime}) must bigger than num ({num})");
            panic!("[FiniteField] Wrong prime");
        }
        if num >= prime {
            let num = num.modpow(&BigInt::one(), &prime);
            FiniteField { num, prime }
        } else {
            FiniteField { num, prime }
        }
    }

    pub fn new(num: BigInt, prime: BigInt) -> Self {
        if num < 1.to_bigint().unwrap() {
            panic!("[FiniteField] num ({num}) must be Natural number")
        }
        FiniteField::raw_new(num, prime)
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

overloading!((lhs : FiniteField) + (rhs : FiniteField) => FiniteField as {
    _lhs_rhs_prime_eq_check!(lhs, rhs);
    FiniteField::raw_new(
        BigInt::modpow(&(&lhs.num + &rhs.num), &BigInt::one(), &lhs.prime),
        lhs.prime.clone()
    )
});
overloading!((lhs : FiniteField) - (rhs : FiniteField) => FiniteField as {
    _lhs_rhs_prime_eq_check!(lhs, rhs);
    FiniteField::raw_new(
        BigInt::modpow(&(&lhs.num - &rhs.num), &BigInt::one(), &lhs.prime),
        lhs.prime.clone(),
    )
});
overloading!((lhs : FiniteField) * (rhs : FiniteField) => FiniteField as {
    _lhs_rhs_prime_eq_check!(lhs, rhs);
    FiniteField::raw_new(
        BigInt::modpow(&(&lhs.num * &rhs.num), &BigInt::one(), &lhs.prime),
        lhs.prime.clone(),
    )
});
overloading!((lhs : FiniteField) / (rhs : FiniteField) => FiniteField as {
    _lhs_rhs_prime_eq_check!(lhs, rhs);
    FiniteField::raw_new(
        BigInt::modpow(&(&lhs.num * &(&rhs.num).mod_inverse(&lhs.prime).unwrap()), &BigInt::one(), &lhs.prime),
        lhs.prime.clone(),
    )
});

overloading!((Pow) (lhs : FiniteField) (pow) (rhs : FiniteField) => FiniteField as {
    _lhs_rhs_prime_eq_check!(lhs, rhs);
    FiniteField::raw_new(
        BigInt::modpow(&lhs.num, &rhs.num, &lhs.prime),
        lhs.prime.clone(),
    )
});

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use num_bigint::ToBigInt;

    use num_bigint_dig::{BigInt, ModInverse};
    use num_traits::{One, Pow};

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
    fn finite_field_new2() {
        let q = FiniteField::new(13.to_bigint().unwrap(), 7.to_bigint().unwrap());
        let res = FiniteField::new(6.to_bigint().unwrap(), 7.to_bigint().unwrap());

        assert_eq!(q, res);
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

    #[test]
    fn multiplicative_inverse_test1() {
        let a1 = FiniteField::new(
            BigInt::from_str("13").unwrap(),
            BigInt::from_str("11").unwrap(),
        );
        let a2 = FiniteField::new(
            BigInt::from_str("14").unwrap(),
            BigInt::from_str("11").unwrap(),
        );
        let res = FiniteField::new(
            BigInt::from_str("8").unwrap(),
            BigInt::from_str("11").unwrap(),
        );
        assert_eq!(a1 / a2, res);
    }

    // #[test]
    // fn generator_test1() {
    //     // let mut gens = HashSet::new();
    //     let mut rng = thread_rng();
    //     let p = rng.gen_prime(2048);
    // }
}
