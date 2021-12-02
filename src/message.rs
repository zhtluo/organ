use rug::Integer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientBaseMessage {
    pub round: usize,
    pub nid: usize,
    pub slot_messages: Vec<Integer>,
    pub slots_needed: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerBaseMessage {
    pub round: usize,
    pub perm: Vec<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientBulkMessage {
    pub round: usize,
    pub nid: usize,
    pub slot_messages: Vec<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    ClientBaseMessage(ClientBaseMessage),
    ServerBaseMessage(ServerBaseMessage),
    ClientBulkMessage(ClientBulkMessage),
}
