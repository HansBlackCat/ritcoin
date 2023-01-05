#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_traits::One;

    #[test]
    fn modpow_test1() {
        let a = BigInt::parse_bytes(b"1233", 10).unwrap();
        let b = BigInt::parse_bytes(b"342", 10).unwrap();
        let c = a.modpow(&BigInt::one(), &b);
        assert_eq!(&c, &BigInt::parse_bytes(b"207", 10).unwrap());
    }
}
