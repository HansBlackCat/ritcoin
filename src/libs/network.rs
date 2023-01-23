pub const BITCOIN_MAINNET_PREFIX: u8 = 0x00_u8;
pub const BITCOIN_TESTNET_PREFIX: u8 = 0x6f_u8;

#[non_exhaustive]
pub enum BitcoinNetwork {
    MainNet,
    TestNet,
}
