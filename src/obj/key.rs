use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    pub raw_key: Vec<u8>,
    pub version: u64,
}

impl Key {
    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

pub fn decode_key(b: &Vec<u8>) -> Key {
    bincode::deserialize(&b).unwrap()
}