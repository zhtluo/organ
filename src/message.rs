use rug::Integer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientBaseMessage {
    pub round: usize,
    pub nid: usize,
    pub slot_messages: Vec<Integer>,
    pub blame: Option<Vec<Integer>>,
    pub blame_blinding: Option<Vec<Integer>>,
    pub e: Option<Vec<Vec<u8>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerBaseMessage {
    pub round: usize,
    pub perm: Vec<Integer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientBulkMessage {
    pub round: usize,
    pub nid: usize,
    pub slot_messages: Vec<Integer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientPrifiMessage {
    pub round: usize,
    pub nid: usize,
    pub slot_messages: Vec<Integer>,
    pub cipher: Integer,
    pub keys: Vec<(Integer, Integer)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    ClientBaseMessage(ClientBaseMessage),
    ServerBaseMessage(ServerBaseMessage),
    ClientBulkMessage(ClientBulkMessage),
    ServerBulkMessage,
    ClientPrifiMessage(ClientPrifiMessage),
    Ok,
}
