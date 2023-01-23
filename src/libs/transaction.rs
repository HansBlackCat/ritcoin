use std::{
    fmt::Display,
    io::{BufReader, Read},
};

use super::script::BitcoinScript;

#[derive(Debug)]
pub struct TransactionVersion {
    version: [u8; 4],
}

impl TransactionVersion {
    pub fn new(version: u32) -> Self {
        TransactionVersion {
            version: u32_to_le_array(version),
        }
    }

    pub fn from_u32(version: u32) -> Self {
        TransactionVersion::new(version)
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        TransactionVersion { version: bytes }
    }

    pub fn to_u32(&self) -> u32 {
        u32::from_le_bytes(self.version)
    }
}

impl Display for TransactionVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex = hex::encode(self.version);
        f.write_str(&hex)
    }
}

#[derive(Debug)]
pub enum TransactionLocktimeType {
    Ignore,
    Block,
    UnixTime,
}

#[derive(Debug)]
pub struct TransactionLocktime {
    locktime: [u8; 4],
    locktime_type: TransactionLocktimeType,
}

impl TransactionLocktime {
    pub fn new(locktime: u32) -> Self {
        let locktime_type = match locktime {
            time if time >= 500_000_000 => TransactionLocktimeType::UnixTime,
            time if time == u32::MAX => TransactionLocktimeType::Ignore,
            _ => TransactionLocktimeType::Block,
        };

        TransactionLocktime {
            locktime: u32_to_le_array(locktime),
            locktime_type,
        }
    }
}

impl Display for TransactionLocktime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(self.locktime))
    }
}

// TODO script_signature -> independent struct
// TODO verification of signature
#[derive(Debug)]
pub struct TransactionInput {
    previous_tx_id: [u8; 32],
    previous_tx_idx: [u8; 4],
    script_signature: Vec<u8>,
    sequence: [u8; 4],
}

impl TransactionInput {
    pub fn new(
        previous_tx_id: [u8; 32],
        previous_tx_idx: [u8; 4],
        script_signature: BitcoinScript,
        sequence: [u8; 4],
    ) -> Self {
        TransactionInput {
            previous_tx_id,
            previous_tx_idx,
            script_signature: script_signature.to_vector(),
            sequence,
        }
    }
}

#[derive(Debug)]
pub struct TransactionOutput {
    amount: Vec<u8>,
    script_pubkey: Vec<u8>,
}

#[derive(Debug)]
pub struct Transaction {
    version: TransactionVersion,
    transaction_inputs_varint: Varint,
    transaction_inputs: Vec<TransactionInput>,
    transaction_outputs_varint: Varint,
    transaction_outputs: Vec<TransactionOutput>,
    locktime: TransactionLocktime,
}

// TODO parse_from_str
impl Transaction {
    pub fn parse(text: &[u8]) {
        let mut stream = BufReader::new(text);
        let mut n: i64 = -1;

        // version (4)
        let array = Self::parse_nbytes::<4>(&mut stream);
    }

    fn parse_nbytes<const N: usize>(stream: &mut BufReader<&[u8]>) -> [u8; N] {
        let mut buffer = [0_u8; N];
        let res = stream.read(&mut buffer).unwrap();
        if res != N {
            panic!("[parse_nbytes] panic while reading nbytes {}", res);
        }
        buffer
    }

    pub fn parse_version(stream: &mut BufReader<&[u8]>) -> TransactionVersion {
        let array = Self::parse_nbytes::<4>(stream);
        TransactionVersion::from_bytes(array)
    }
}

pub fn u32_to_le_array(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

#[derive(Debug)]
pub struct Varint(Vec<u8>);

impl Varint {
    pub fn new(value: u64) -> Self {
        Varint(varint_representation(value))
    }
}

pub fn varint_representation(value: u64) -> Vec<u8> {
    let mut vec: Vec<u8> = Vec::new();
    match value {
        v if v < 253 => vec.extend((value as u8).to_le_bytes()),
        v if v < (1 << 16) => {
            vec.push(0xfd_u8);
            vec.extend((value as u16).to_le_bytes())
        }
        v if v < (1 << 32) => {
            vec.push(0xfe_u8);
            vec.extend((value as u32).to_le_bytes())
        }
        _ => {
            vec.push(0xff_u8);
            vec.extend(value.to_le_bytes())
        }
    }
    vec
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read};

    use super::{varint_representation, Transaction, TransactionVersion};

    const TX1: &str = "010000000456919960ac691763688d3d3bcea9ad6ecaf875df5339e\
    148a1fc61c6ed7a069e010000006a47304402204585bcdef85e6b1c6af5c2669d4830ff86e42dd\
    205c0e089bc2a821657e951c002201024a10366077f87d6bce1f7100ad8cfa8a064b39d4e8fe4e\
    a13a7b71aa8180f012102f0da57e85eec2934a82a585ea337ce2f4998b50ae699dd79f5880e253\
    dafafb7feffffffeb8f51f4038dc17e6313cf831d4f02281c2a468bde0fafd37f1bf882729e7fd\
    3000000006a47304402207899531a52d59a6de200179928ca900254a36b8dff8bb75f5f5d71b1c\
    dc26125022008b422690b8461cb52c3cc30330b23d574351872b7c361e9aae3649071c1a716012\
    1035d5c93d9ac96881f19ba1f686f15f009ded7c62efe85a872e6a19b43c15a2937feffffff567\
    bf40595119d1bb8a3037c356efd56170b64cbcc160fb028fa10704b45d775000000006a4730440\
    2204c7c7818424c7f7911da6cddc59655a70af1cb5eaf17c69dadbfc74ffa0b662f02207599e08\
    bc8023693ad4e9527dc42c34210f7a7d1d1ddfc8492b654a11e7620a0012102158b46fbdff65d0\
    172b7989aec8850aa0dae49abfb84c81ae6e5b251a58ace5cfeffffffd63a5e6c16e620f86f375\
    925b21cabaf736c779f88fd04dcad51d26690f7f345010000006a47304402200633ea0d3314bea\
    0d95b3cd8dadb2ef79ea8331ffe1e61f762c0f6daea0fabde022029f23b3e9c30f080446150b23\
    852028751635dcee2be669c2a1686a4b5edf304012103ffd6f4a67e94aba353a00882e563ff272\
    2eb4cff0ad6006e86ee20dfe7520d55feffffff0251430f00000000001976a914ab0c0b2e98b1a\
    b6dbf67d4750b0a56244948a87988ac005a6202000000001976a9143c82d7df364eb6c75be8c80\
    df2b3eda8db57397088ac46430600";

    #[test]
    fn tx_version() {
        let v1 = TransactionVersion::new(1);
        assert_eq!(v1.version, [1, 0, 0, 0]);
        assert_eq!(v1.to_string(), "01000000");
    }

    #[test]
    fn variants_test() {
        assert_eq!(varint_representation(100), vec![0x64_u8]);
        assert_eq!(varint_representation(255), vec![0xfd_u8, 0xff, 0x00]);
        assert_eq!(varint_representation(555), vec![0xfd_u8, 0x2b, 0x02]);
        assert_eq!(
            varint_representation(70015),
            vec![0xfe_u8, 0x7f, 0x11, 0x01, 0x00]
        );
        assert_eq!(
            varint_representation(18005558675309),
            vec![0xff_u8, 0x6d, 0xc7, 0xed, 0x3e, 0x60, 0x10, 0x00, 0x00]
        );
    }

    #[test]
    fn bufreader_test() {
        let text = vec![0x21, 0x20, 0x19, 0x18, 0x17_u8];
        let text = text.as_slice();
        let mut stream = BufReader::new(text);

        let mut buf = [0_u8; 1];
        let mut s = stream.read(&mut buf).unwrap();
        assert_eq!(buf, [0x21_u8]);
        assert_eq!(s, 1);
        s = stream.read(&mut buf).unwrap();
        assert_eq!(s, 1);
        assert_eq!(buf, [0x20_u8]);
        s = stream.read(&mut buf).unwrap();
        assert_eq!(buf, [0x19_u8]);
        assert_eq!(s, 1);
    }

    #[test]
    fn parse_version() {
        let hex = hex::decode(TX1).unwrap();
        let hex = hex.as_slice();
        let mut stream = BufReader::new(hex);
        let v = Transaction::parse_version(&mut stream);
        eprintln!("{:?}", v.version);
        assert_eq!(v.to_u32(), 1_u32);
    }
}
