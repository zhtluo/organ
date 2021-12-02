use crate::config::{Config, MAX_MESSAGE_SIZE};
use crate::message::{ClientBaseMessage, ClientBulkMessage, Message};
use rug::Integer;
use std::net::UdpSocket;

fn send_client_base_message(
    c: &Config,
    nid: usize,
    base_prf: &Vec<Integer>,
    socket: &UdpSocket,
    round: usize,
) {
    debug!("p: {}", c.base_params.p);
    debug!("q: {}", c.base_params.q);
    debug!("num_of_slots: {}", c.client_addr.len());
    debug!(
        "evaluations: [{}, {}, {}, ...]",
        base_prf[0], base_prf[1], base_prf[2]
    );

    let mut slot_msg = Integer::from(1);
    let mut slot_messages = Vec::<Integer>::with_capacity(c.client_addr.len());
    let message_ele = Integer::from(nid + 1);
    for i in 0..c.client_addr.len() {
        slot_msg = slot_msg * &message_ele;
        slot_msg = slot_msg % &c.base_params.p;
        let msg_to_append = Integer::from(&base_prf[i] + 1000 * &slot_msg) % &c.base_params.q;
        slot_messages.push(msg_to_append);
    }

    let message = bincode::serialize(&Message::ClientBaseMessage(ClientBaseMessage {
        round: round,
        nid: nid,
        slot_messages: slot_messages,
        slots_needed: 1,
    }))
    .unwrap();

    info!("Sending ClientBaseMessage, size = {}...", message.len());
    socket.send_to(&message, c.server_addr).unwrap();
    info!("Sent ClientBaseMessage.");
}

fn send_client_bulk_message(
    c: &Config,
    nid: usize,
    bulk_prf: &Vec<Integer>,
    socket: &UdpSocket,
    round: usize,
) {
    let message = bincode::serialize(&Message::ClientBulkMessage(ClientBulkMessage {
        round: round,
        nid: nid,
        slot_messages: bulk_prf.clone(),
    }))
    .unwrap();
    info!("Sending ClientBulkMessage, size = {}...", message.len());
    socket.send_to(&message, c.server_addr).unwrap();
    info!("Sent ClientBulkMessage.");
}

pub fn main(c: Config, nid: usize, base_prf: Vec<Integer>, bulk_prf: Vec<Integer>) {
    let socket = UdpSocket::bind(c.client_addr[nid]).unwrap();
    let mut round: usize = 0;
    loop {
        round += 1;
        info!("Round {}.", round);
        send_client_base_message(&c, nid, &base_prf, &socket, round);

        let mut buf = [0; MAX_MESSAGE_SIZE];
        let (size, _src) = socket.recv_from(&mut buf).unwrap();
        let message: Message = bincode::deserialize(&buf[..size]).unwrap();
        match message {
            Message::ServerBaseMessage(msg) => {
                info!("Received ServerBaseMessage on round {}.", msg.round);
                if msg.round == round {
                    send_client_bulk_message(&c, nid, &bulk_prf, &socket, round);
                }
            }
            _ => {
                error!("Unknown message {:?}.", message);
            }
        }
    }
}
