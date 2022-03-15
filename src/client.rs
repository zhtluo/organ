use crate::config::Config;
use crate::guard::SetupValues;
use crate::message::{ClientBaseMessage, ClientBulkMessage, ClientPrifiMessage, Message};
use crate::net::{read_stream, write_stream};
use rug::Integer;
use std::net::TcpStream;

/// Add randomness to generate the cipher text for the base round.
pub fn generate_client_base_message(
    c: &Config,
    prf: &Vec<Integer>,
    message_ele: &Integer,
) -> Vec<Integer> {
    let mut slot_msg = Integer::from(1);
    let mut slot_messages = Vec::<Integer>::with_capacity(c.client_size);
    for i in 0..c.client_size {
        slot_msg = slot_msg * message_ele;
        slot_msg = slot_msg % &c.base_params.p;
        let msg_to_append = Integer::from(&prf[i] + 1000 * &slot_msg) % &c.base_params.q;
        slot_messages.push(msg_to_append);
    }
    slot_messages
}

/// Process and send the base round message.
fn send_client_base_message(
    c: &Config,
    nid: usize,
    base_prf: &SetupValues,
    message_ele: &Integer,
    socket: &mut TcpStream,
    round: usize,
) {
    let mut scaled = base_prf.share.scaled.clone();
    // If we need to compute PRF on-demand...
    if c.do_unzip {
        scaled = crate::prf::compute(&c.base_params, &base_prf);
    }
    let message = bincode::serialize(&Message::ClientBaseMessage(ClientBaseMessage {
        round: round,
        nid: nid,
        slot_messages: generate_client_base_message(&c, &scaled, message_ele),
        blame: if c.do_blame {
            Some(base_prf.share.scaled.clone())
        } else {
            None
        },
        blame_blinding: if c.do_blame {
            Some(base_prf.blinding.scaled.clone())
        } else {
            None
        },
        e: base_prf.e.clone(),
    }))
    .unwrap();

    // Sleep to mesaure the optimal round trip time.
    if c.do_delay && nid == 0 {
        std::thread::sleep(std::time::Duration::from_secs(2))
    }

    info!("Sending ClientBaseMessage, size = {}...", message.len());
    write_stream(socket, &message).unwrap();
    info!("Sent ClientBaseMessage.");
}

/// Process and send the bulk round message.
fn send_client_bulk_message(
    c: &Config,
    nid: usize,
    posid: usize,
    bulk_prf: &SetupValues,
    socket: &mut TcpStream,
    round: usize,
) {
    let mut scaled = bulk_prf.share.scaled.clone();
    // If we need to compute PRF on-demand...
    if c.do_unzip {
        scaled = crate::prf::compute(&c.bulk_params, &bulk_prf);
    }
    let slot_index_start = posid * c.slot_per_round;
    let slot_index_end = (posid + 1) * c.slot_per_round;
    let mut prf_evaluations = scaled[0..c.slot_per_round * c.client_size].to_vec();
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

    // Sleep to mesaure the optimal round trip time.
    if c.do_delay && nid == 0 {
        std::thread::sleep(std::time::Duration::from_secs(2))
    }

    info!("Sending ClientBulkMessage, size = {}...", message.len());
    write_stream(socket, &message).unwrap();
    info!("Sent ClientBulkMessage.");
}

/// Overarching function.
pub fn main(c: Config, nid: usize, base_prf: SetupValues, bulk_prf: SetupValues) {
    debug!("Connecting to {:?}...", c.server_addr);
    let mut socket = TcpStream::connect(c.server_addr).unwrap();
    let mut round: usize = 0;
    let mut rand = rug::rand::RandState::new();
    rand.seed(&base_prf.share.scaled[0]);
    loop {
        if round < c.round {
            round += 1;
            info!("Round {}.", round);

            // Generate a random number for identification.
            let message_ele = Integer::from(c.base_params.p.random_below_ref(&mut rand));
            info!("Message in base round: {}", message_ele);
            send_client_base_message(&c, nid, &base_prf, &message_ele, &mut socket, round);

            let buf = read_stream(&mut socket).unwrap();
            let message: Message = bincode::deserialize(&buf).unwrap();
            match message {
                Message::ServerBaseMessage(msg) => {
                    info!("Received ServerBaseMessage on round {}.", msg.round);
                    if msg.round == round {
                        send_client_bulk_message(
                            &c,
                            nid,
                            msg.perm.iter().position(|x| x == &message_ele).unwrap(),
                            &bulk_prf,
                            &mut socket,
                            round,
                        );
                    }
                }
                _ => {
                    error!("Unknown message {:?}.", message);
                }
            }

            let buf = read_stream(&mut socket).unwrap();
            let message: Message = bincode::deserialize(&buf).unwrap();
            match message {
                Message::ServerBulkMessage => {
                    info!("Received ServerBulkMessage.");
                }
                _ => {
                    error!("Unknown message {:?}.", message);
                }
            }
        } else {
            // Sleep a little bit after everything finishes to ensure that the message is sent.
            std::thread::sleep(std::time::Duration::from_secs(5));
            return;
        }
    }
}

/// Code to time Prifi.
pub fn main_prifi(c: Config, nid: usize) {
    debug!("Connecting to {:?}...", c.server_addr);
    let mut socket = TcpStream::connect(c.server_addr).unwrap();
    let mut round: usize = 0;
    loop {
        if round < c.round {
            round += 1;
            info!("Round {}.", round);
            let nbits: usize = c.slot_per_round * 8;
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
            if c.do_delay && nid == 0 {
                std::thread::sleep(std::time::Duration::from_secs(2))
            }
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
            std::thread::sleep(std::time::Duration::from_secs(5));
            return;
        }
    }
}
