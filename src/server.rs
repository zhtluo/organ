use crate::config::{Config, MAX_MESSAGE_SIZE};
use crate::message::{ClientBaseMessage, ClientBulkMessage, Message, ServerBaseMessage};
use rug::Integer;
use std::collections::HashMap;
use std::net::UdpSocket;

fn solve_equation(
    c: &Config,
    base_prf: &Vec<Integer>,
    messages: &HashMap<usize, ClientBaseMessage>,
) -> Vec<Integer> {
    let mut relay_messages = Vec::<Integer>::with_capacity(messages.len());
    for (_nid, msg) in messages.iter() {
        let mut relay_msg_of_slot = Integer::from(0);
        for smsg in msg.slot_messages.iter() {
            relay_msg_of_slot = (relay_msg_of_slot + smsg) % &c.base_params.q;
        }
        relay_messages.push(relay_msg_of_slot);
    }
    debug!("relay_messages: {:?}", relay_messages);

    let mut final_equations = Vec::<Integer>::with_capacity(relay_messages.len());
    for (rmsg, prf) in relay_messages.iter().zip(base_prf.iter()) {
        let val = (Integer::from(rmsg - prf) / 1000) % &c.base_params.q;
        let val_in_grp = val % &c.base_params.p;
        final_equations.push(val_in_grp);
    }
    debug!("final_equations: {:?}", final_equations);

    // Use some dummy values for now.
    vec![Integer::from(1); c.client_addr.len()]
}

pub fn main(c: Config, base_prf: Vec<Integer>, _bulk_prf: Vec<Integer>) {
    let socket = UdpSocket::bind(c.server_addr).unwrap();
    let mut base_protocol_buffer = HashMap::<usize, HashMap<usize, ClientBaseMessage>>::new();
    let mut bulk_protocol_buffer = HashMap::<usize, HashMap<usize, ClientBulkMessage>>::new();
    let mut round: usize = 0;
    loop {
        round += 1;
        info!("Round {}.", round);
        if base_protocol_buffer.get(&round).is_none() {
            base_protocol_buffer.insert(round, HashMap::new());
        }
        if bulk_protocol_buffer.get(&round).is_none() {
            bulk_protocol_buffer.insert(round, HashMap::new());
        }

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
                    if base_protocol_buffer.get(&msg.round).is_none() {
                        base_protocol_buffer.insert(msg.round, HashMap::new());
                    }
                    base_protocol_buffer
                        .get_mut(&msg.round)
                        .unwrap()
                        .insert(msg.nid, msg);
                }
                Message::ClientBulkMessage(msg) => {
                    info!(
                        "Received ClientBulkMessage from {} on round {}.",
                        msg.nid, msg.round
                    );
                    if bulk_protocol_buffer.get(&msg.round).is_none() {
                        bulk_protocol_buffer.insert(msg.round, HashMap::new());
                    }
                    bulk_protocol_buffer
                        .get_mut(&msg.round)
                        .unwrap()
                        .insert(msg.nid, msg);
                }
                _ => {
                    error!("Unknown message {:?}.", message);
                }
            }

            if base_protocol_buffer.get(&round).unwrap().len() == c.client_addr.len() {
                info!("All base messages received. Computing...");
                let perm =
                    solve_equation(&c, &base_prf, &base_protocol_buffer.get(&round).unwrap());
                let message = bincode::serialize(&Message::ServerBaseMessage(ServerBaseMessage {
                    round: round,
                    perm: perm,
                }))
                .unwrap();
                info!("Sending ServerBaseMessage, size = {}...", message.len());
                for addr in c.client_addr.iter() {
                    socket.send_to(&message, addr).unwrap();
                }
                info!("Sent ServerBaseMessage.");
                base_protocol_buffer.remove(&round);
            }

            if bulk_protocol_buffer.get(&round).unwrap().len() == c.client_addr.len() {
                info!("All bulk messages received. Computing...");
                bulk_protocol_buffer.remove(&round);
                break;
            }
        }
    }
}
