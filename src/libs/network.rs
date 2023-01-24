use digest::Digest;
use sha2::Sha256;

pub const BITCOIN_MAINNET_PREFIX: u8 = 0x00_u8;
pub const BITCOIN_TESTNET_PREFIX: u8 = 0x6f_u8;

pub const BITCOIN_NETWORK_MAGIC_FLAG_MAINNET: [u8; 4] = [0xf9, 0xbe, 0xb4, 0xd9];
pub const BITCOIN_NETWORK_MAGIC_FLAG_TESTNET: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];

#[non_exhaustive]
pub enum BitcoinNetwork {
    MainNet,
    TestNet,
}

pub struct NetworkMagic {
    magic: [u8; 4],
}

pub struct NetworkCommand {
    command: [u8; 12],
}

pub struct NetworkPayload {
    payload_length: [u8; 4],
    payload_checksum: [u8; 4],
    payload: Vec<u8>,
}

impl NetworkPayload {
    pub fn new(payload: Vec<u8>) -> NetworkPayload {
        let length: [u8; 4] = u32::to_le_bytes(payload.len() as u32);

        let _checksum = Sha256::digest(&payload).to_vec();
        let checksum: [u8; 4] = _checksum
            .as_slice()
            .try_into()
            .expect("fail to cast to array");

        NetworkPayload {
            payload_length: length,
            payload_checksum: checksum,
            payload,
        }
    }
}

pub struct Network {
    pub network_magic: NetworkMagic,
    pub command: NetworkCommand,
    pub payload: NetworkPayload,
}
