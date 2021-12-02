use rug::integer::ParseIntegerError;
use rug::ops::Pow;
use rug::Integer;
use std::net::{AddrParseError, SocketAddr};

pub const MAX_MESSAGE_SIZE: usize = 1024;

#[derive(Clone, Debug)]
pub struct ProtocolParams {
    pub p: Integer,
    pub q: Integer,
    pub ring_v: Integer,
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
    pub client_addr: Vec<SocketAddr>,
    pub base_params: ProtocolParams,
    pub bulk_params: ProtocolParams,
}

#[derive(Debug)]
pub enum ConfigError {
    AddrParseError(AddrParseError),
    ParseIntegerError(ParseIntegerError),
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

pub fn load_config() -> Result<Config, ConfigError> {
    Ok(Config {
        server_addr: "127.0.0.1:8001".parse()?,
        client_addr: vec!["127.0.0.1:9001".parse()?, "127.0.0.1:9002".parse()?],
        base_params: ProtocolParams {
            p: Integer::from(2).pow(32) - 5,
            // order of secp112r1
            q: Integer::from_str_radix("db7c2abf62e35e7628dfac6561c5", 16)?,
            ring_v: (Integer::from(57) * (Integer::from(2).pow(96))) + 1,
            vector_len: 2048,
            bits: 32,
        },
        bulk_params: ProtocolParams {
            p: Integer::from(2).pow(226) - 5,
            // order of secp112r1
            q: Integer::from_str_radix("db7c2abf62e35e7628dfac6561c5", 16)?,
            ring_v: (Integer::from(7) * (Integer::from(2).pow(290))) + 1,
            vector_len: 8192,
            bits: 226,
        },
    })
}
