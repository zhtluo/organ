use rug::Integer;
use serde::{Deserialize, Serialize};

/// Client base round message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientBaseMessage {
    /// Round number.
    pub round: usize,
    /// Client ID.
    pub nid: usize,
    /// Client message.
    pub slot_messages: Vec<Integer>,
    /// Opening of the share in the blame protocol message.
    pub blame: Option<Vec<Integer>>,
    /// Opening of the blinding vector in the blame protocol message.
    pub blame_blinding: Option<Vec<Integer>>,
    /// Value `e` in the blame protocol message.
    pub e: Option<Vec<Vec<u8>>>,
}

/// Server base round message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerBaseMessage {
    /// Round number.
    pub round: usize,
    /// Permutation of client-generated one-time IDs.
    pub perm: Vec<Integer>,
}

/// Client bulk round message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientBulkMessage {
    /// Round number.
    pub round: usize,
    /// Client ID.
    pub nid: usize,
    /// Client message.
    pub slot_messages: Vec<Integer>,
}

/// Client message during the PriFi protocol, used in timing.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientPrifiMessage {
    /// Round number.
    pub round: usize,
    /// Client ID.
    pub nid: usize,
    /// Client message.
    pub slot_messages: Vec<Integer>,
    /// Cipher.
    pub cipher: Integer,
    /// Keys.
    pub keys: Vec<(Integer, Integer)>,
}

/// Message used in the protocol.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// Client base round message.
    ClientBaseMessage(ClientBaseMessage),
    /// Server base round message.
    ServerBaseMessage(ServerBaseMessage),
    /// Client bulk round message.
    ClientBulkMessage(ClientBulkMessage),
    /// Server bulk round message.
    ServerBulkMessage,
    /// Client message during the PriFi protocol, used in timing.
    ClientPrifiMessage(ClientPrifiMessage),
    /// Server OK message during the PriFi protocol, used in timing.
    Ok,
}
