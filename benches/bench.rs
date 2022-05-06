use criterion::BenchmarkId;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use openssl::{ec::EcGroup, nid::Nid};
use organ::config::*;
use organ::*;
use rug::ops::Pow;
use rug::Integer;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

fn get_config(client_size: usize, slot: usize) -> config::Config {
    config::Config {
        server_addr: std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ),
        client_size,
        base_params: ProtocolParams {
            p: Integer::from(2).pow(64) - 59,
            q: Integer::from(2).pow(84) - 35,
            ring_v: NttField {
                order: (Integer::from(57) * (Integer::from(2).pow(96))) + 1,
                root: Integer::from_str_radix("2418184924512328812370262861594", 10).unwrap(),
                scale: Integer::from(2).pow(96),
            },
            vector_len: 2048,
            bits: 32,
            group_nid: Nid::SECP256K1.as_raw(),
            group: Some(EcGroup::from_curve_name(Nid::SECP256K1).unwrap()),
        },
        bulk_params: ProtocolParams {
            p: Integer::from(2).pow(226) - 5,
            // order of secp256k1
            q: Integer::from_str_radix(
                "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
                16,
            )
            .unwrap(),
            ring_v: NttField {
                order: (Integer::from(7) * (Integer::from(2).pow(290))) + 1,
                root: Integer::from_str_radix("2187", 10).unwrap(),
                scale: Integer::from(2).pow(290),
            },
            vector_len: 8192,
            bits: 226,
            group_nid: Nid::SECT571K1.as_raw(),
            group: Some(EcGroup::from_curve_name(Nid::SECT571K1).unwrap()),
        },
        do_blame: false,
        do_unzip: false,
        do_delay: false,
        do_ping: false,
        slot_per_round: slot,
        round: 0,
    }
}

fn get_setup_relay(
    client_size: usize,
    params: &ProtocolParams,
) -> (Vec<guard::SetupValues>, guard::SetupRelay) {
    let shares: Vec<Vec<Integer>> = (0..params.vector_len)
        .map(|_| guard::generate_sum_shares(client_size, &params.ring_v.order, &Integer::from(1)))
        .collect();
    let shares: Vec<Vec<Integer>> = (0..client_size)
        .map(|i| shares.iter().map(|v| v[i].clone()).collect())
        .collect();
    let setup_values: Vec<guard::SetupValues> = (0..client_size)
        .map(|i| guard::gen_setup_values(params, &shares[i], false))
        .collect();
    let setup_relay = guard::gen_setup_relay(params, &setup_values, false);

    (setup_values, setup_relay)
}

pub fn criterion_benchmark_solve_eq(cr: &mut Criterion) {
    let mut group = cr.benchmark_group("solve_eq_single");
    for size in [50, 100, 150, 200].iter() {
        let c = get_config(*size, 3);
        let (sv, sr) = get_setup_relay(*size, &c.base_params);
        let mut messages =
            std::collections::HashMap::<usize, crate::message::ClientBaseMessage>::new();
        for (i, v) in sv.iter().enumerate() {
            messages.insert(
                i,
                message::ClientBaseMessage {
                    round: 0,
                    nid: i,
                    slot_messages: client::generate_client_base_message(
                        &c,
                        &v.share.scaled,
                        &Integer::from(i),
                    ),
                    blame: None,
                    blame_blinding: None,
                    e: None,
                },
            );
        }
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| server::solve_equation(&c, &sr.values.share.scaled, &messages));
        });
    }
    group.finish();
    let mut group = cr.benchmark_group("solve_eq_multi");
    for size in [50, 100, 150, 200].iter() {
        let c = get_config(*size, 3);
        let (sv, sr) = get_setup_relay(*size, &c.base_params);
        let mut messages =
            std::collections::HashMap::<usize, crate::message::ClientBaseMessage>::new();
        for (i, v) in sv.iter().enumerate() {
            messages.insert(
                i,
                message::ClientBaseMessage {
                    round: 0,
                    nid: i,
                    slot_messages: client::generate_client_base_message(
                        &c,
                        &v.share.scaled,
                        &Integer::from(i),
                    ),
                    blame: None,
                    blame_blinding: None,
                    e: None,
                },
            );
        }
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let c = get_config(size, 3);
                const NTHREADS: u32 = 100;
                let mut children = vec![];
                let (tx, rx) = mpsc::channel();
                let param_arc = Arc::new((c, sr.clone(), messages.clone()));
                for _ in 0..NTHREADS {
                    let txc = tx.clone();
                    let paramc = Arc::clone(&param_arc);
                    children.push(thread::spawn(move || {
                        txc.send(server::solve_equation(
                            &paramc.0,
                            &paramc.1.values.share.scaled,
                            &paramc.2,
                        ))
                        .unwrap();
                    }));
                }
                for _ in 0..NTHREADS {
                    rx.recv().unwrap();
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark_solve_eq);

criterion_main!(benches);
