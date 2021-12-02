use crate::config::{Config, MAX_MESSAGE_SIZE};
use crate::message::{ClientBaseMessage, Message};
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

pub fn main(c: Config) {
    let socket = UdpSocket::bind(c.server_addr).unwrap();
    let mut base_protocol_buffer = HashMap::<usize, ClientBaseMessage>::new();
    let mut round: usize = 0;
    loop {
        round += 1;
        info!("Round: {}", round);

        loop {
            let mut buf = [0; MAX_MESSAGE_SIZE];
            let (size, _src) = socket.recv_from(&mut buf).unwrap();
            let message: Message = bincode::deserialize(&buf[..size]).unwrap();
            match message {
                Message::ClientBaseMessage(msg) => {
                    info!(
                        "Received ClientBaseMessage from {} on round {}.",
                        msg.nid, msg.round
                    );
                    if msg.round == round {
                        base_protocol_buffer.insert(msg.nid, msg);
                    }
                }
            }

            if base_protocol_buffer.len() == c.client_addr.len() {
                info!("All messages received.");
                base_protocol_buffer.clear();
                break;
            }
        }
    }
}
