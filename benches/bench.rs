use criterion::{black_box, criterion_group, criterion_main, Criterion};
use organ::config::*;
use organ::*;
use rug::ops::Pow;
use rug::Integer;
use std::collections::HashMap;

pub fn criterion_benchmark_compute_message(c: &mut Criterion) {
    let conf = config::Config {
        server_addr: std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ),
        client_size: 200,
        base_params: ProtocolParams {
            p: Integer::from(2).pow(32) - 5,
            // order of secp112r1
            q: Integer::from_str_radix("db7c2abf62e35e7628dfac6561c5", 16).unwrap(),
            ring_v: NttField {
                order: (Integer::from(57) * (Integer::from(2).pow(96))) + 1,
                root: Integer::from_str_radix("2418184924512328812370262861594", 10).unwrap(),
                scale: Integer::from(2).pow(96),
            },
            vector_len: 2048,
            bits: 32,
        },
        bulk_params: ProtocolParams {
            p: Integer::from(2).pow(226) - 5,
            // order of secp256k1
            q: Integer::from_str_radix(
                "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
                16,
            ).unwrap(),
            ring_v: NttField {
                order: (Integer::from(7) * (Integer::from(2).pow(290))) + 1,
                root: Integer::from_str_radix("2187", 10).unwrap(),
                scale: Integer::from(2).pow(290),
            },
            vector_len: 8192,
            bits: 226,
        },
        round: 0,
    };
    let mut rand = rug::rand::RandState::new();
    let bulk: Vec<Integer> =
        std::iter::repeat_with(|| Integer::from(conf.bulk_params.p.random_below_ref(&mut rand)))
            .take(conf.bulk_params.vector_len * conf.client_size)
            .collect();
    let mut messages = HashMap::<usize, organ::message::ClientBulkMessage>::new();
    for i in 0..conf.client_size {
        messages.insert(
            i,
            organ::message::ClientBulkMessage {
                nid: i,
                round: 1,
                slot_messages: std::iter::repeat_with(|| {
                    Integer::from(conf.bulk_params.p.random_below_ref(&mut rand))
                })
                .take(conf.bulk_params.vector_len * conf.client_size)
                .collect(),
            },
        );
    }
    c.bench_function("compute_message", |b| {
        b.iter(|| {
            server::compute_message(black_box(&conf), black_box(&bulk), black_box(&messages));
        })
    });
}

criterion_group!(benches, criterion_benchmark_compute_message);
criterion_main!(benches);
