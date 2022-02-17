use crate::config::Config;
use crate::guard::SetupValues;
use crate::message::{ClientBaseMessage, ClientBulkMessage, ClientPrifiMessage, Message};
use crate::net::{read_stream, write_stream};
use rug::Integer;
use std::net::TcpStream;

fn generate_client_base_message(c: &Config, nid: usize, prf: &Vec<Integer>) -> Vec<Integer> {
    let mut slot_msg = Integer::from(1);
    let mut slot_messages = Vec::<Integer>::with_capacity(c.client_size);
    let message_ele = Integer::from(nid + 1);
    for i in 0..c.client_size {
        slot_msg = slot_msg * &message_ele;
        slot_msg = slot_msg % &c.base_params.p;
        let msg_to_append = Integer::from(&prf[i] + 1000 * &slot_msg) % &c.base_params.q;
        slot_messages.push(msg_to_append);
    }
    slot_messages
}

fn send_client_base_message(
    c: &Config,
    nid: usize,
    base_prf: &SetupValues,
    socket: &mut TcpStream,
    round: usize,
) {
    debug!("p: {}", c.base_params.p);
    debug!("q: {}", c.base_params.q);
    debug!("num_of_slots: {}", c.client_size);
    debug!(
        "evaluations: [{}, {}, {}, ...]",
        base_prf.share.scaled[0], base_prf.share.scaled[1], base_prf.share.scaled[2]
    );
    /*
    let mut rand = rug::rand::RandState::new();
    crate::timing::compute(&crate::timing::CompParameters {
        a: std::iter::repeat_with(|| Integer::from(c.base_params.p.random_below_ref(&mut rand)))
            .take(c.base_params.vector_len)
            .collect(),
        b: std::iter::repeat_with(|| Integer::from(c.base_params.p.random_below_ref(&mut rand)))
            .take(c.base_params.vector_len)
            .collect(),
        p: c.base_params.p.clone(),
        w: c.base_params.ring_v.clone(),
        order: c.base_params.q.clone(),
    });
    */

    let message = bincode::serialize(&Message::ClientBaseMessage(ClientBaseMessage {
        round: round,
        nid: nid,
        slot_messages: generate_client_base_message(&c, nid, &base_prf.share.scaled),
        blame: base_prf.share.scaled.clone(),
        blame_blinding: base_prf.blinding.scaled.clone(),
        slots_needed: 1,
        e: base_prf.e.clone(),
    }))
    .unwrap();

    info!("Sending ClientBaseMessage, size = {}...", message.len());
    write_stream(socket, &message).unwrap();
    info!("Sent ClientBaseMessage.");
}

fn send_client_bulk_message(
    c: &Config,
    nid: usize,
    bulk_prf: &SetupValues,
    socket: &mut TcpStream,
    round: usize,
) {
    /*
    let mut rand = rug::rand::RandState::new();
    crate::timing::compute(&crate::timing::CompParameters {
        a: std::iter::repeat_with(|| Integer::from(c.base_params.p.random_below_ref(&mut rand)))
            .take((c.bulk_params.vector_len * c.client_size).next_power_of_two())
            .collect(),
        b: std::iter::repeat_with(|| Integer::from(c.base_params.p.random_below_ref(&mut rand)))
            .take((c.bulk_params.vector_len * c.client_size).next_power_of_two())
            .collect(),
        p: c.bulk_params.p.clone(),
        w: c.bulk_params.ring_v.clone(),
        order: c.bulk_params.q.clone(),
    });
    */

    let slots_per_client = c.bulk_params.vector_len / c.client_size;
    let slot_index_start = nid * slots_per_client;
    let slot_index_end = (nid + 1) * slots_per_client;
    let mut prf_evaluations = bulk_prf.share.scaled.clone();
    // prf_evaluations.resize(slots_per_client * c.client_size, Integer::from(0));
    let message_ele = nid + 1;
    for i in slot_index_start..slot_index_end {
        prf_evaluations[i] =
            (&prf_evaluations[i] + Integer::from(1000 * message_ele)) % &c.bulk_params.q;
    }
    let message = bincode::serialize(&Message::ClientBulkMessage(ClientBulkMessage {
        round: round,
        nid: nid,
        slot_messages: prf_evaluations,
    }))
    .unwrap();
    info!("Sending ClientBulkMessage, size = {}...", message.len());
    write_stream(socket, &message).unwrap();
    info!("Sent ClientBulkMessage.");
}

pub fn main(c: Config, nid: usize, base_prf: SetupValues, bulk_prf: SetupValues) {
    debug!("Connecting to {:?}...", c.server_addr);
    let mut socket = TcpStream::connect(c.server_addr).unwrap();
    let mut round: usize = 0;
    loop {
        if round < c.round {
            round += 1;
            info!("Round {}.", round);
            send_client_base_message(&c, nid, &base_prf, &mut socket, round);

            let buf = read_stream(&mut socket).unwrap();
            let message: Message = bincode::deserialize(&buf).unwrap();
            match message {
                Message::ServerBaseMessage(msg) => {
                    info!("Received ServerBaseMessage on round {}.", msg.round);
                    if msg.round == round {
                        send_client_bulk_message(&c, nid, &bulk_prf, &mut socket, round);
                    }
                }
                _ => {
                    error!("Unknown message {:?}.", message);
                }
            }
        } else {
            return;
        }
    }
}

pub fn main_prifi(c: Config, nid: usize, _base_prf: Vec<Integer>, _bulk_prf: Vec<Integer>) {
    debug!("Connecting to {:?}...", c.server_addr);
    let mut socket = TcpStream::connect(c.server_addr).unwrap();
    let mut round: usize = 0;
    loop {
        if round < c.round {
            round += 1;
            info!("Round {}.", round);
            let nbits: usize = 1024 * 8;
            let nguards: usize = 10;
            let mut rand = rug::rand::RandState::new();
            let prgs: Vec<Integer> = std::iter::repeat_with(|| {
                Integer::from(Integer::random_bits(nbits as u32, &mut rand))
            })
            .take(nguards)
            .collect();
            let message = Integer::from(Integer::random_bits(nbits as u32, &mut rand));
            let slot_messages: Vec<Integer> = std::iter::repeat_with(|| message.clone())
                .take(c.client_size)
                .collect();
            let keys = vec![(Integer::from_str_radix("c90fdaa22168c234c4c6628b80dc1cd129024e088a67cc74020bbea63b139b22514a08798e3404ddef9519b3cd3a431b302b0a6df25f14374fe1356d6d51c5ef", 16).unwrap(), Integer::from_str_radix("c90fdaa22168c234c4c6628b80dc1cd129024e088a67cc74020bbea63b139b22514a08798e3404ddef9519b3cd3a431b302b0a6df25f14374fe1356d6d51c5ef", 16).unwrap()); c.client_size];
            let mut message_enc = message ^ &prgs[0];
            for i in 2..nguards {
                message_enc ^= &prgs[i];
            }
            let message = bincode::serialize(&Message::ClientPrifiMessage(ClientPrifiMessage {
                round: round,
                nid: nid,
                slot_messages: slot_messages,
                keys: keys,
                cipher: message_enc,
            }))
            .unwrap();
            info!("Sending ClientPrifiMessage, size = {}...", message.len());
            write_stream(&mut socket, &message).unwrap();
            info!("Sent ClientPrifiMessage.");
            let buf = read_stream(&mut socket).unwrap();
            let message: Message = bincode::deserialize(&buf).unwrap();
            match message {
                Message::Ok => {
                    info!("Received Server Ok Message.");
                }
                _ => {
                    error!("Unknown message {:?}.", message);
                }
            }
        } else {
            return;
        }
    }
}
