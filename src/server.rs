use crate::config::Config;
use crate::flint::solve_impl;
use crate::message::{ClientBaseMessage, ClientBulkMessage, Message, ServerBaseMessage};
use crate::net::{async_read_stream, async_write_stream};
use async_std::channel::{unbounded, Receiver, Sender};
use async_std::net::{TcpListener, TcpStream};
use futures::stream::StreamExt;
use futures::{future::join_all, join, select, FutureExt};
use rug::Integer;
use std::collections::HashMap;

fn solve_equation(
    c: &Config,
    base_prf: &Vec<Integer>,
    messages: &HashMap<usize, ClientBaseMessage>,
) -> Vec<Integer> {
    let mut relay_messages = Vec::<Integer>::with_capacity(c.base_params.vector_len);
    for i in 0..c.client_size {
        let mut relay_msg_of_slot = Integer::from(0);
        for (_nid, msg) in messages.iter() {
            relay_msg_of_slot = (relay_msg_of_slot + &msg.slot_messages[i]) % &c.base_params.q;
        }
        relay_messages.push(relay_msg_of_slot);
    }
    debug!("base_relay_messages: {:?}", relay_messages);

    let mut final_equations = Vec::<Integer>::with_capacity(relay_messages.len());
    for (rmsg, prf) in relay_messages.iter().zip(base_prf.iter()) {
        let val =
            (Integer::from(rmsg - prf) % &c.base_params.q + &c.base_params.q) % &c.base_params.q;
        let val_in_grp = val / 1000 % &c.base_params.p;
        final_equations.push(val_in_grp);
    }
    debug!("final_equations: {:?}", final_equations);

    let solve = solve_impl(&c.base_params.p, &final_equations);
    debug!("solve: {:?}", solve);

    solve
}

fn compute_message(
    c: &Config,
    bulk_prf: &Vec<Integer>,
    messages: &HashMap<usize, ClientBulkMessage>,
) -> Vec<Integer> {
    let mut relay_messages = Vec::<Integer>::with_capacity(c.bulk_params.vector_len);
    for i in 0..c.bulk_params.vector_len {
        let mut relay_msg_of_slot = Integer::from(0);
        for (_nid, msg) in messages.iter() {
            relay_msg_of_slot = (relay_msg_of_slot + &msg.slot_messages[i]) % &c.bulk_params.q;
        }
        relay_messages.push(relay_msg_of_slot);
    }
    debug!("bulk_relay_messages: {:?}", relay_messages);

    let mut final_values = Vec::<Integer>::with_capacity(relay_messages.len());
    for (rmsg, prf) in relay_messages.iter().zip(bulk_prf.iter()) {
        let val =
            (Integer::from(rmsg - prf) % &c.bulk_params.q + &c.bulk_params.q) % &c.bulk_params.q;
        let val_in_grp = val / 1000 % &c.bulk_params.p;
        final_values.push(val_in_grp);
    }
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
                let msg = read_result.unwrap();
                channel_read.send(msg).await.unwrap();
            }
            write_result = channel_write.recv().fuse() => {
                let message = write_result.unwrap();
                async_write_stream(&mut stream, &message).await.unwrap();
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

pub async fn main(c: Config, base_prf: Vec<Integer>, bulk_prf: Vec<Integer>) {
    let (boardcast_channels_send, boardcast_channels_recv) = unbounded::<Sender<Vec<u8>>>();
    let (reactor_input_channel_send, reactor_input_channel_recv) = unbounded::<Vec<u8>>();
    let (reactor_output_channel_send, reactor_output_channel_recv) = unbounded::<Vec<u8>>();
    join!(
        listener(&c, reactor_input_channel_send, boardcast_channels_send),
        sender(boardcast_channels_recv, reactor_output_channel_recv),
        reactor(
            &c,
            base_prf,
            bulk_prf,
            reactor_input_channel_recv,
            reactor_output_channel_send
        ),
    );
}

pub async fn reactor(
    c: &Config,
    base_prf: Vec<Integer>,
    bulk_prf: Vec<Integer>,
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
    join!(
        msg_dist(),
        reactor_base_round(
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
        )
    );
}

pub async fn reactor_base_round(
    c: &Config,
    base_prf: Vec<Integer>,
    base_input_channel: Receiver<ClientBaseMessage>,
    reactor_output_channel: Sender<Vec<u8>>,
) {
    let mut chan = Vec::with_capacity(c.round);
    for _ in 0..c.round {
        chan.push(unbounded::<ClientBaseMessage>());
    }
    let mut futures = Vec::with_capacity(c.round);
    for i in 0..c.round {
        futures.push(reactor_base_one_round(
            c,
            &base_prf,
            &chan[i].1,
            &reactor_output_channel,
            i,
        ));
    }
    join!(
        join_all(futures),
        reactor_base_round_listener(&base_input_channel, &chan)
    );
}

pub async fn reactor_base_round_listener(
    base_input_channel: &Receiver<ClientBaseMessage>,
    base_output_channel: &Vec<(Sender<ClientBaseMessage>, Receiver<ClientBaseMessage>)>,
) {
    loop {
        let msg = base_input_channel.recv().await.unwrap();
        info!(
            "Received ClientBaseMessage from {} on round {}.",
            msg.nid, msg.round
        );
        base_output_channel[msg.round].0.send(msg).await.unwrap();
    }
}

pub async fn reactor_base_one_round(
    c: &Config,
    base_prf: &Vec<Integer>,
    base_input_channel: &Receiver<ClientBaseMessage>,
    reactor_output_channel: &Sender<Vec<u8>>,
    round: usize,
) {
    let mut base_protocol_buffer = HashMap::<usize, ClientBaseMessage>::new();
    loop {
        let msg = base_input_channel.recv().await.unwrap();
        base_protocol_buffer.insert(msg.nid, msg);
        if base_protocol_buffer.len() == c.client_size {
            info!("All base messages received. Computing...");
            let perm = solve_equation(&c, &base_prf, &base_protocol_buffer);
            let message = bincode::serialize(&Message::ServerBaseMessage(ServerBaseMessage {
                round: round,
                perm: perm,
            }))
            .unwrap();
            info!("Sending ServerBaseMessage, size = {}...", message.len());
            reactor_output_channel.send(message).await.unwrap();
            info!("Sent ServerBaseMessage.");
            break;
        }
    }
}

pub async fn reactor_bulk_round(
    c: &Config,
    bulk_prf: Vec<Integer>,
    bulk_input_channel: Receiver<ClientBulkMessage>,
    reactor_output_channel: Sender<Vec<u8>>,
) {
    let mut chan = Vec::with_capacity(c.round);
    for _ in 0..c.round {
        chan.push(unbounded::<ClientBulkMessage>());
    }
    let mut futures = Vec::with_capacity(c.round);
    for i in 0..c.round {
        futures.push(reactor_bulk_one_round(
            c,
            &bulk_prf,
            &chan[i].1,
            &reactor_output_channel,
            i,
        ));
    }
    join!(
        async {
            join_all(futures).await;
            panic!("Time up.");
        },
        reactor_bulk_round_listener(&bulk_input_channel, &chan)
    );
}

pub async fn reactor_bulk_round_listener(
    bulk_input_channel: &Receiver<ClientBulkMessage>,
    bulk_output_channel: &Vec<(Sender<ClientBulkMessage>, Receiver<ClientBulkMessage>)>,
) {
    loop {
        let msg = bulk_input_channel.recv().await.unwrap();
        info!(
            "Received ClientBulkMessage from {} on round {}.",
            msg.nid, msg.round
        );
        bulk_output_channel[msg.round].0.send(msg).await.unwrap();
    }
}

pub async fn reactor_bulk_one_round(
    c: &Config,
    bulk_prf: &Vec<Integer>,
    bulk_input_channel: &Receiver<ClientBulkMessage>,
    reactor_output_channel: &Sender<Vec<u8>>,
    _round: usize,
) {
    let mut bulk_protocol_buffer = HashMap::<usize, ClientBulkMessage>::new();
    loop {
        let msg = bulk_input_channel.recv().await.unwrap();
        bulk_protocol_buffer.insert(msg.nid, msg);
        if bulk_protocol_buffer.len() == c.client_size {
            info!("All bulk messages received. Computing...");
            compute_message(c, &bulk_prf, &bulk_protocol_buffer);
            let message = bincode::serialize(&Message::ServerBulkMessage).unwrap();
            info!("Sending ServerBulkMessage, size = {}...", message.len());
            reactor_output_channel.send(message).await.unwrap();
            info!("Sent ServerBulkMessage.");
            break;
        }
    }
}

/*
pub async fn reactor_bulk_round(
    c: &Config,
    bulk_prf: Vec<Integer>,
    bulk_input_channel: Receiver<ClientBulkMessage>,
    _reactor_output_channel: Sender<Vec<u8>>,
) {
    let mut bulk_protocol_buffer = HashMap::<usize, HashMap<usize, ClientBulkMessage>>::new();
    let mut round: usize = 0;
    loop {
        round += 1;
        info!("Bulk round {}.", round);
        if round > c.round {
            panic!("Time up.")
        }
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
                compute_message(c, &bulk_prf, &bulk_protocol_buffer.get(&round).unwrap());
                bulk_protocol_buffer.remove(&round);
                break;
            }
        }
    }
}
*/

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
