use crate::config::{Config, ProtocolParams};
use crate::ecc::{add, from_bytes, get_g, get_h, mul, new_big_num_context};
use crate::flint::solve_impl;
use crate::guard::SetupRelay;
use crate::message::{
    ClientBaseMessage, ClientBulkMessage, ClientPrifiMessage, Message, ServerBaseMessage,
};
use crate::net::{async_read_stream, async_write_stream};
use async_std::channel::{unbounded, Receiver, Sender};
use async_std::net::{TcpListener, TcpStream};
use futures::stream::StreamExt;
use futures::{future::join, select, FutureExt};
use rayon::prelude::*;
use rug::{Complete, Integer};
use std::collections::HashMap;

pub fn solve_equation(
    c: &Config,
    base_prf: &SetupRelay,
    messages: &HashMap<usize, ClientBaseMessage>,
) -> Vec<Integer> {
    debug!("Client messages: {:?}", messages);

    let relay_messages: Vec<Integer> = (0..c.client_size)
        .into_par_iter()
        .map(|i| messages.par_iter().map(|(_, b)| &b.slot_messages[i]).sum())
        .collect();
    debug!("base_relay_messages: {:?}", relay_messages);

    let final_values: Vec<Integer> = relay_messages
        .par_iter()
        .zip(base_prf.values.share.scaled.par_iter())
        .map(|(rmsg, prf)| {
            let (_quotient, rem) = Integer::from(rmsg - prf)
                .div_rem_euc_ref(&c.base_params.q)
                .complete();
            rem
        })
        .collect();
    debug!("final_values before rounding: {:?}", final_values);
    let final_values: Vec<Integer> = final_values
        .par_iter()
        .map(|x| (x + Integer::from(1000 / 2)) / 1000 % &c.base_params.p)
        .collect();
    debug!("final_values: {:?}", final_values);

    let solve = solve_impl(&c.base_params.p, &final_values);
    debug!("solve: {:?}", solve);

    solve
}

pub fn compute_message(
    c: &Config,
    bulk_prf: &SetupRelay,
    messages: &HashMap<usize, ClientBulkMessage>,
) -> Vec<Integer> {
    let relay_messages: Vec<Integer> = (0..c.slot_per_round * c.client_size)
        .into_par_iter()
        .map(|i| {
            let mut relay_msg_of_slot = Integer::from(0);
            for (_nid, msg) in messages.iter() {
                relay_msg_of_slot = (relay_msg_of_slot + &msg.slot_messages[i]) % &c.bulk_params.q;
            }
            relay_msg_of_slot
        })
        .collect();

    /*
    let mut futures = Vec::new();
    for i in 0..c.bulk_params.vector_len * c.client_size {
        futures.push((|| async move {
            let mut relay_msg_of_slot = Integer::from(0);
            for (_nid, msg) in messages.iter() {
                relay_msg_of_slot = (relay_msg_of_slot + &msg.slot_messages[i]) % &c.bulk_params.q;
            }
            relay_msg_of_slot
        })());
    }
    let relay_messages = join_all(futures).await;
    debug!("bulk_relay_messages: {:?}", relay_messages);
    */

    let final_values: Vec<Integer> = relay_messages
        .par_iter()
        .zip(bulk_prf.values.share.scaled.par_iter())
        .map(|(rmsg, prf)| {
            (Integer::from(rmsg - prf) % &c.bulk_params.q + &c.bulk_params.q) % &c.bulk_params.q
        })
        .collect();
    debug!("final_values before rounding: {:?}", final_values);
    let final_values: Vec<Integer> = final_values
        .par_iter()
        .map(|x| (x + Integer::from(1000 / 2)) / 1000 % &c.bulk_params.p)
        .collect();
    debug!("final_values: {:?}", final_values);

    final_values
}

async fn handle_connection(
    mut stream: TcpStream,
    channel_read: Sender<Vec<u8>>,
    channel_write: Receiver<Vec<u8>>,
) {
    loop {
        select! {
            read_result = async_read_stream(&mut stream).fuse() => {
                if read_result.is_ok() {
                    let msg = read_result.unwrap();
                    channel_read.send(msg).await.unwrap();
                }
            }
            write_result = channel_write.recv().fuse() => {
                if write_result.is_ok() {
                    let message = write_result.unwrap();
                    if async_write_stream(&mut stream, &message).await.is_err() {
                        error!("Write error on socket.");
                    }
                }
            }
        }
    }
}

async fn listener(
    c: &Config,
    reactor_input_channel_send: Sender<Vec<u8>>,
    boardcast_channels_send: Sender<Sender<Vec<u8>>>,
) {
    let listener = TcpListener::bind(c.server_addr).await.unwrap();
    listener
        .incoming()
        .for_each_concurrent(None, |stream| {
            let boardcast_channels_send = boardcast_channels_send.clone();
            let reactor_input_channel_send = reactor_input_channel_send.clone();
            async move {
                let stream = stream.unwrap();
                let (channel_send, channel_recv) = unbounded::<Vec<u8>>();
                boardcast_channels_send.send(channel_send).await.unwrap();
                handle_connection(stream, reactor_input_channel_send, channel_recv).await;
            }
        })
        .await;
}

async fn sender(
    boardcast_channels_recv: Receiver<Sender<Vec<u8>>>,
    reactor_output_channel_recv: Receiver<Vec<u8>>,
) {
    let mut channels = Vec::<Sender<Vec<u8>>>::new();
    loop {
        select! {
            new_channel = boardcast_channels_recv.recv().fuse() => {
                let new_channel = new_channel.unwrap();
                channels.push(new_channel);
            }
            new_message = reactor_output_channel_recv.recv().fuse() => {
                let new_message = new_message.unwrap();
                for chan in channels.iter() {
                    chan.send(new_message.clone()).await.unwrap();
                }
            }
        }
    }
}

pub async fn main(c: Config, base_prf: SetupRelay, bulk_prf: SetupRelay) {
    let (boardcast_channels_send, boardcast_channels_recv) = unbounded::<Sender<Vec<u8>>>();
    let (reactor_input_channel_send, reactor_input_channel_recv) = unbounded::<Vec<u8>>();
    let (reactor_output_channel_send, reactor_output_channel_recv) = unbounded::<Vec<u8>>();
    select!(
        () = listener(&c, reactor_input_channel_send, boardcast_channels_send).fuse() => {},
        () = sender(boardcast_channels_recv, reactor_output_channel_recv).fuse() => {},
        () = reactor(
            &c,
            base_prf,
            bulk_prf,
            reactor_input_channel_recv,
            reactor_output_channel_send
        ).fuse() => {
            debug!("Main finished.");
        }
    );
}

pub async fn main_prifi(c: Config) {
    let (boardcast_channels_send, boardcast_channels_recv) = unbounded::<Sender<Vec<u8>>>();
    let (reactor_input_channel_send, reactor_input_channel_recv) = unbounded::<Vec<u8>>();
    let (reactor_output_channel_send, reactor_output_channel_recv) = unbounded::<Vec<u8>>();
    select!(
        () = listener(&c, reactor_input_channel_send, boardcast_channels_send).fuse() => {},
        () = sender(boardcast_channels_recv, reactor_output_channel_recv).fuse() => {},
        () = reactor_prifi(&c, reactor_input_channel_recv, reactor_output_channel_send).fuse() => {}
    );
}

pub async fn reactor_prifi(
    c: &Config,
    reactor_input_channel: Receiver<Vec<u8>>,
    reactor_output_channel: Sender<Vec<u8>>,
) {
    let (base_input_channel_send, base_input_channel_recv) = unbounded::<ClientPrifiMessage>();
    let msg_dist = || async move {
        loop {
            let message = &reactor_input_channel.recv().await.unwrap();
            info!("Got message of size {}.", message.len());
            let message: Message = bincode::deserialize(&message).unwrap();
            match message {
                Message::ClientPrifiMessage(msg) => {
                    base_input_channel_send.send(msg).await.unwrap();
                }
                _ => {
                    error!("Unknown message {:?}.", message);
                }
            }
        }
    };
    select!(
        () = msg_dist().fuse() => {},
        () = reactor_prifi_round(c, base_input_channel_recv, reactor_output_channel.clone()).fuse() => {}
    );
}

pub async fn reactor_prifi_round(
    c: &Config,
    base_input_channel: Receiver<ClientPrifiMessage>,
    reactor_output_channel: Sender<Vec<u8>>,
) {
    let mut base_protocol_buffer = HashMap::<usize, HashMap<usize, ClientPrifiMessage>>::new();
    let mut round: usize = 0;
    loop {
        round += 1;
        if round > c.round {
            info!("Base round finished.");
            async_std::task::sleep(std::time::Duration::from_secs(5)).await;
            return;
        }
        info!("Base round {}.", round);
        if base_protocol_buffer.get(&round).is_none() {
            base_protocol_buffer.insert(round, HashMap::new());
        }
        loop {
            let msg = base_input_channel.recv().await.unwrap();
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
            if base_protocol_buffer.get(&round).unwrap().len() == c.client_size {
                info!("All prifi messages received. Computing...");
                let nbits: usize = c.slot_per_round * 8;
                let nguards: usize = 10;
                let mut rand = rug::rand::RandState::new();
                let prgs: Vec<Integer> = std::iter::repeat_with(|| {
                    Integer::from(Integer::random_bits(nbits as u32, &mut rand))
                })
                .take(nguards)
                .collect();
                for i in 0..c.client_size {
                    let msg = base_protocol_buffer.get(&round).unwrap().get(&i).unwrap();
                    let mut xored_val = msg.slot_messages[0].clone();
                    let mut xored_prg = Integer::from(0);
                    for j in 2..c.client_size {
                        xored_val = xored_val
                            ^ &base_protocol_buffer
                                .get(&round)
                                .unwrap()
                                .get(&j)
                                .unwrap()
                                .slot_messages[i];
                    }
                    for k in 2..nguards {
                        xored_prg = xored_prg ^ &prgs[k];
                    }
                    xored_val ^= xored_prg;
                }
                info!(
                    "{}",
                    std::str::from_utf8(
                        &std::process::Command::new("ping")
                            .arg("google.com")
                            .arg("-c")
                            .arg("1")
                            .output()
                            .unwrap()
                            .stdout
                    )
                    .unwrap()
                );
                let message = bincode::serialize(&Message::Ok).unwrap();
                info!("Sending Server Ok Message, size = {}...", message.len());
                reactor_output_channel.send(message).await.unwrap();
                info!("Sent Server Ok Message.");
                base_protocol_buffer.remove(&round);
                break;
            }
        }
    }
}

pub async fn reactor(
    c: &Config,
    base_prf: SetupRelay,
    bulk_prf: SetupRelay,
    reactor_input_channel: Receiver<Vec<u8>>,
    reactor_output_channel: Sender<Vec<u8>>,
) {
    let (base_input_channel_send, base_input_channel_recv) = unbounded::<ClientBaseMessage>();
    let (bulk_input_channel_send, bulk_input_channel_recv) = unbounded::<ClientBulkMessage>();
    let msg_dist = || async move {
        loop {
            let message = &reactor_input_channel.recv().await.unwrap();
            info!("Got message of size {}.", message.len());
            let message: Message = bincode::deserialize(&message).unwrap();
            match message {
                Message::ClientBaseMessage(msg) => {
                    base_input_channel_send.send(msg).await.unwrap();
                }
                Message::ClientBulkMessage(msg) => {
                    bulk_input_channel_send.send(msg).await.unwrap();
                }
                _ => {
                    error!("Unknown message {:?}.", message);
                }
            }
        }
    };
    select!(
        () = msg_dist().fuse() => {},
        ((), ()) = join(reactor_base_round(
            c,
            base_prf,
            base_input_channel_recv,
            reactor_output_channel.clone()
        ),
        reactor_bulk_round(
            c,
            bulk_prf,
            bulk_input_channel_recv,
            reactor_output_channel.clone()
        )).fuse() => {
            debug!("Reactor finished.");
        }
    );
}

fn verify(
    params: &ProtocolParams,
    msg: &Vec<Integer>,
    msg_b: &Vec<Integer>,
    e: &Vec<Vec<u8>>,
    qw: &Vec<Vec<u8>>,
) -> bool {
    msg.par_iter()
        .zip(msg_b.par_iter())
        .zip(e.par_iter())
        .zip(qw.par_iter())
        .all(|(((a, b), c), d)| {
            add(
                params,
                &add(
                    params,
                    &mul(
                        params,
                        &get_g(params),
                        &Integer::from(a * &params.ring_v.order),
                    ),
                    &mul(
                        params,
                        &get_h(params),
                        &Integer::from(b * &params.ring_v.order),
                    ),
                ),
                &from_bytes(params, c),
            )
            .eq(
                &params.group.as_ref().unwrap(),
                &from_bytes(params, d),
                &mut new_big_num_context(),
            )
            .unwrap()
        })
}

pub async fn reactor_base_round(
    c: &Config,
    base_prf: SetupRelay,
    base_input_channel: Receiver<ClientBaseMessage>,
    reactor_output_channel: Sender<Vec<u8>>,
) {
    let mut base_protocol_buffer = HashMap::<usize, HashMap<usize, ClientBaseMessage>>::new();
    let mut round: usize = 0;
    loop {
        round += 1;
        if round > c.round {
            info!("Base round finished.");
            return;
        }
        info!("Base round {}.", round);
        if base_protocol_buffer.get(&round).is_none() {
            base_protocol_buffer.insert(round, HashMap::new());
        }
        loop {
            let msg = base_input_channel.recv().await.unwrap();
            info!(
                "Received ClientBaseMessage from {} on round {}.",
                msg.nid, msg.round
            );
            if base_protocol_buffer.get(&msg.round).is_none() {
                base_protocol_buffer.insert(msg.round, HashMap::new());
            }
            if c.do_blame {
                if !verify(
                    &c.base_params,
                    &msg.blame.as_ref().unwrap(),
                    &msg.blame_blinding.as_ref().unwrap(),
                    msg.e.as_ref().unwrap(),
                    &base_prf.qw.as_ref().unwrap()[msg.nid],
                ) {
                    warn!("Blame protocol verification failure for {}.", msg.nid);
                }
            }
            base_protocol_buffer
                .get_mut(&msg.round)
                .unwrap()
                .insert(msg.nid, msg);
            if base_protocol_buffer.get(&round).unwrap().len() == c.client_size {
                info!("All base messages received. Computing...");
                if c.do_unzip {
                    let mut rand = rug::rand::RandState::new();
                    crate::timing::compute(&crate::timing::CompParameters {
                        a: std::iter::repeat_with(|| {
                            Integer::from(c.base_params.p.random_below_ref(&mut rand))
                        })
                        .take(c.base_params.vector_len)
                        .collect(),
                        b: std::iter::repeat_with(|| {
                            Integer::from(c.base_params.p.random_below_ref(&mut rand))
                        })
                        .take(c.base_params.vector_len)
                        .collect(),
                        p: c.base_params.p.clone(),
                        w: c.base_params.ring_v.order.clone(),
                        order: c.base_params.q.clone(),
                    });
                }
                let perm =
                    solve_equation(&c, &base_prf, &base_protocol_buffer.get(&round).unwrap());
                let message = bincode::serialize(&Message::ServerBaseMessage(ServerBaseMessage {
                    round: round,
                    perm: perm,
                }))
                .unwrap();
                info!("Sending ServerBaseMessage, size = {}...", message.len());
                reactor_output_channel.send(message).await.unwrap();
                info!("Sent ServerBaseMessage.");
                base_protocol_buffer.remove(&round);
                break;
            }
        }
    }
}

pub async fn reactor_bulk_round(
    c: &Config,
    bulk_prf: SetupRelay,
    bulk_input_channel: Receiver<ClientBulkMessage>,
    reactor_output_channel: Sender<Vec<u8>>,
) {
    let mut bulk_protocol_buffer = HashMap::<usize, HashMap<usize, ClientBulkMessage>>::new();
    let mut round: usize = 0;
    loop {
        round += 1;
        if round > c.round {
            info!("Bulk round finished.");
            async_std::task::sleep(std::time::Duration::from_secs(5)).await;
            return;
        }
        info!("Bulk round {}.", round);
        if bulk_protocol_buffer.get(&round).is_none() {
            bulk_protocol_buffer.insert(round, HashMap::new());
        }
        loop {
            let msg = bulk_input_channel.recv().await.unwrap();
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
            if bulk_protocol_buffer.get(&round).unwrap().len() == c.client_size {
                info!("All bulk messages received. Computing...");
                if c.do_unzip {
                    let mut rand = rug::rand::RandState::new();
                    crate::timing::compute(&crate::timing::CompParameters {
                        a: std::iter::repeat_with(|| {
                            Integer::from(c.base_params.p.random_below_ref(&mut rand))
                        })
                        .take(c.bulk_params.vector_len.next_power_of_two())
                        .collect(),
                        b: std::iter::repeat_with(|| {
                            Integer::from(c.base_params.p.random_below_ref(&mut rand))
                        })
                        .take(c.bulk_params.vector_len.next_power_of_two())
                        .collect(),
                        p: c.bulk_params.p.clone(),
                        w: c.bulk_params.ring_v.order.clone(),
                        order: c.bulk_params.q.clone(),
                    });
                }
                compute_message(c, &bulk_prf, &bulk_protocol_buffer.get(&round).unwrap());
                if c.do_ping {
                    info!(
                        "{}",
                        std::str::from_utf8(
                            &std::process::Command::new("ping")
                                .arg("google.com")
                                .arg("-c")
                                .arg("1")
                                .output()
                                .unwrap()
                                .stdout
                        )
                        .unwrap()
                    );
                }
                let message = bincode::serialize(&Message::ServerBulkMessage).unwrap();
                info!("Sending ServerBulkMessage, size = {}...", message.len());
                reactor_output_channel.send(message).await.unwrap();
                info!("Sent ServerBulkMessage.");
                bulk_protocol_buffer.remove(&round);
                break;
            }
        }
    }
}

/*
pub async fn main(c: Config, base_prf: Vec<Integer>, _bulk_prf: Vec<Integer>) {
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

            if base_protocol_buffer.get(&round).unwrap().len() == c.client_size {
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

            if bulk_protocol_buffer.get(&round).unwrap().len() == c.client_size {
                info!("All bulk messages received. Computing...");
                bulk_protocol_buffer.remove(&round);
                break;
            }
        }
    }
}
*/
