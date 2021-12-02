use crate::config::{Config, MAX_MESSAGE_SIZE};
use crate::message::{ClientBaseMessage, Message};
use rug::Integer;
use std::net::{SocketAddr, UdpSocket};

fn send_client_base_message(
    c: &Config,
    nid: usize,
    base_prf: &Vec<Integer>,
    socket: &UdpSocket,
    round: usize,
) {
    info!("p: {}", c.base_params.p);
    info!("q: {}", c.base_params.q);
    info!("num_of_slots: {}", c.client_addr.len());
    info!(
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

    info!("Sending ClientBaseMessage...");
    socket.send_to(&message, c.server_addr).unwrap();
    info!("Sent ClientBaseMessage.");
}

pub fn main(c: Config, nid: usize, base_prf: Vec<Integer>, bulk_prf: Vec<Integer>) {
    let socket = UdpSocket::bind(c.client_addr[nid]).unwrap();
    let mut round: usize = 0;
    loop {
        round += 1;
        info!("Round: {}", round);
        send_client_base_message(&c, nid, &base_prf, &socket, round);

        let mut buf = [0; MAX_MESSAGE_SIZE];
        let (size, _src) = socket.recv_from(&mut buf).unwrap();
        let _message: Message = bincode::deserialize(&buf[..size]).unwrap();
    }
}
