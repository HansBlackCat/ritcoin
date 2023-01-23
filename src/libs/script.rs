pub struct BitcoinScript {}

impl BitcoinScript {
    pub fn to_vector(&self) -> Vec<u8> {
        vec![0]
    }
}
