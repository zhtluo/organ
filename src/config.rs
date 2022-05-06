use openssl::{ec::EcGroup, nid::Nid};
use rug::integer::ParseIntegerError;
use rug::ops::Pow;
use rug::Integer;
use serde::{Deserialize, Serialize};
use std::net::{AddrParseError, SocketAddr};

/// Protocol parameters for one round. (base or bulk)
#[derive(Serialize, Deserialize)]
pub struct ProtocolParams {
    /// Value of `p`.
    pub p: Integer,
    /// Value of `q`.
    pub q: Integer,
    /// Value of `v`.
    pub ring_v: NttField,
    /// Length of the vector.
    pub vector_len: usize,
    /// Total number of bits.
    pub bits: usize,
    /// ECC group id, specified by OpenSSL.
    pub group_nid: i32,
    /// ECC group.
    #[serde(skip)]
    pub group: Option<EcGroup>,
}

/// Config for the protocol.
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Server address.
    pub server_addr: SocketAddr,
    /// Number of clients in the system.
    pub client_size: usize,
    /// Base protocol parameters.
    #[serde(default = "default_base_params")]
    pub base_params: ProtocolParams,
    /// Bulk protocol parameters.
    #[serde(default = "default_bulk_params")]
    pub bulk_params: ProtocolParams,
    /// How many rounds to run.
    pub round: usize,
    /// How many slots a client needs in the bulk round.
    pub slot_per_round: usize,
    /// Whether or not to test the blame protocol.
    #[serde(default)]
    pub do_blame: bool,
    /// Whether or not to generate PRF on-demand.
    #[serde(default)]
    pub do_unzip: bool,
    /// Whether or not to simulate a delay to compute optimal round-trip time.
    #[serde(default)]
    pub do_delay: bool,
    /// Whether ot not to simulate a ping to WWW.
    #[serde(default)]
    pub do_ping: bool,
}

/// Config-related error.
#[derive(Debug)]
pub enum ConfigError {
    /// Parse error.
    AddrParseError(AddrParseError),
    /// Parse error.
    ParseIntegerError(ParseIntegerError),
    /// IO error.
    IOError(std::io::Error),
    /// JSON-related error.
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

/// A field for NTT operations.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NttField {
    /// The order of the field.
    pub order: Integer,
    /// The primitive root of the field.
    pub root: Integer,
    /// The power of `root` so that `root.pow_mod(scale, order) == 1`.
    pub scale: Integer,
}

impl NttField {
    /// Find the root with respect to `n`, i.e. `root.pow_mod(n, order) == 1`.
    pub fn root_of_unity(&self, n: usize) -> Integer {
        self.root
            .clone()
            .pow_mod(&(self.scale.clone() / Integer::from(n)), &self.order)
            .unwrap()
    }
}

/// Returns default base parameters.
fn default_base_params() -> ProtocolParams {
    ProtocolParams {
        p: Integer::from(2).pow(64) - 59,
        q: Integer::from(2).pow(84) - 35,
        ring_v: NttField {
            order: (Integer::from(57) * (Integer::from(2).pow(96))) + 1,
            root: Integer::from_str_radix("2418184924512328812370262861594", 10).unwrap(),
            scale: Integer::from(2).pow(96),
        },
        vector_len: 2048,
        bits: 64,
        group_nid: Nid::SECP256K1.as_raw(),
        group: Some(EcGroup::from_curve_name(Nid::SECP256K1).unwrap()),
    }
}

/// Returns default bulk parameters.
fn default_bulk_params() -> ProtocolParams {
    ProtocolParams {
        p: Integer::from(2).pow(226) - 5,
        // order of secp256k1
        q: Integer::from_str_radix(
            "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
            16,
        )
        .unwrap(),
        ring_v: NttField {
            order: (Integer::from(7) * (Integer::from(2).pow(290))) + 1,
            root: Integer::from_str_radix("2187", 10).unwrap(),
            scale: Integer::from(2).pow(290),
        },
        vector_len: 8192,
        bits: 226,
        group_nid: Nid::SECT571K1.as_raw(),
        group: Some(EcGroup::from_curve_name(Nid::SECT571K1).unwrap()),
    }
}

/// Loads config from a file.
pub fn load_config(filename: &str) -> Result<Config, ConfigError> {
    let mut c: Config = serde_json::from_str(&std::fs::read_to_string(filename)?)?;
    c.base_params.group =
        Some(EcGroup::from_curve_name(Nid::from_raw(c.base_params.group_nid)).unwrap());
    c.bulk_params.group =
        Some(EcGroup::from_curve_name(Nid::from_raw(c.bulk_params.group_nid)).unwrap());
    Ok(c)
}

/// Dumps the current config into a file.
pub fn dump_config(filename: &str, c: &Config) -> Result<(), ConfigError> {
    std::fs::write(&filename, serde_json::to_string_pretty(&c).unwrap())?;
    Ok(())
}
