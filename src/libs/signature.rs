use num_bigint_dig::BigUint;

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
}
