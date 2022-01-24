use rug::integer::ParseIntegerError;
use rug::ops::Pow;
use rug::Integer;
use serde::{Deserialize, Serialize};
use std::net::{AddrParseError, SocketAddr};

#[derive(Clone, Debug)]
pub struct ProtocolParams {
    pub p: Integer,
    pub q: Integer,
    pub ring_v: NttField,
    pub vector_len: usize,
    pub bits: usize,
}

#[derive(Clone, Debug)]
pub struct SlotParams {
    pub fragments: usize,
    pub slot_number: usize,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub server_addr: SocketAddr,
    pub client_size: usize,
    pub base_params: ProtocolParams,
    pub bulk_params: ProtocolParams,
    pub round: usize,
}

#[derive(Debug)]
pub enum ConfigError {
    AddrParseError(AddrParseError),
    ParseIntegerError(ParseIntegerError),
    IOError(std::io::Error),
    JsonError(serde_json::Error),
}

impl From<AddrParseError> for ConfigError {
    fn from(e: AddrParseError) -> Self {
        ConfigError::AddrParseError(e)
    }
}

impl From<ParseIntegerError> for ConfigError {
    fn from(e: ParseIntegerError) -> Self {
        ConfigError::ParseIntegerError(e)
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::IOError(e)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(e: serde_json::Error) -> Self {
        ConfigError::JsonError(e)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonConfig {
    pub server_addr: SocketAddr,
    pub client_size: usize,
    pub round: usize,
}

#[derive(Clone, Debug)]
pub struct NttField {
    pub order: Integer,
    pub root: Integer,
    pub scale: Integer,
}

impl NttField {
    pub fn root_of_unity(&self, n: usize) -> Integer {
        self.root
            .clone()
            .pow_mod(&(self.scale.clone() / Integer::from(n)), &self.order)
            .unwrap()
    }
}

pub fn load_config(filename: &str) -> Result<Config, ConfigError> {
    let cf: JsonConfig = serde_json::from_str(&std::fs::read_to_string(filename)?)?;
    Ok(Config {
        server_addr: cf.server_addr,
        client_size: cf.client_size,
        base_params: ProtocolParams {
            p: Integer::from(2).pow(32) - 5,
            // order of secp112r1
            q: Integer::from_str_radix("db7c2abf62e35e7628dfac6561c5", 16)?,
            ring_v: NttField {
                order: (Integer::from(57) * (Integer::from(2).pow(96))) + 1,
                root: Integer::from_str_radix("2418184924512328812370262861594", 10)?,
                scale: Integer::from(2).pow(96),
            },
            vector_len: 2048,
            bits: 32,
        },
        bulk_params: ProtocolParams {
            p: Integer::from(2).pow(226) - 5,
            // order of secp256k1
            q: Integer::from_str_radix(
                "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
                16,
            )?,
            ring_v: NttField {
                order: (Integer::from(7) * (Integer::from(2).pow(290))) + 1,
                root: Integer::from_str_radix("2187", 10)?,
                scale: Integer::from(2).pow(290),
            },
            vector_len: 8192,
            bits: 226,
        },
        round: cf.round,
    })
}
