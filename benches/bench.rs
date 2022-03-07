use criterion::BenchmarkId;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use openssl::{ec::EcGroup, nid::Nid};
use organ::config::*;
use organ::*;
use rug::ops::Pow;
use rug::Integer;

pub fn criterion_benchmark_compute_prf_bulk(cr: &mut Criterion) {
    let c = config::Config {
        server_addr: std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ),
        client_size: 200,
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
        slot_per_round: 3,
        round: 0,
    };
    let mut group = cr.benchmark_group("compute_prf_bulk");
    for size in [50, 100, 150, 200].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut rand = rug::rand::RandState::new();
                crate::timing::compute(&crate::timing::CompParameters {
                    a: std::iter::repeat_with(|| {
                        Integer::from(c.base_params.p.random_below_ref(&mut rand))
                    })
                    .take((c.slot_per_round * size).next_power_of_two())
                    .collect(),
                    b: std::iter::repeat_with(|| {
                        Integer::from(c.base_params.p.random_below_ref(&mut rand))
                    })
                    .take((c.slot_per_round * size).next_power_of_two())
                    .collect(),
                    p: c.bulk_params.p.clone(),
                    w: c.bulk_params.ring_v.order.clone(),
                    order: c.bulk_params.q.clone(),
                });
            });
        });
    }
    group.finish();
}

pub fn criterion_benchmark_compute_prf_base(cr: &mut Criterion) {
    let c = config::Config {
        server_addr: std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ),
        client_size: 200,
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
        slot_per_round: 3,
        round: 0,
    };
    let mut group = cr.benchmark_group("compute_prf_base");
    for size in [50, 100, 150, 200].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut rand = rug::rand::RandState::new();
                crate::timing::compute(&crate::timing::CompParameters {
                    a: std::iter::repeat_with(|| {
                        Integer::from(c.base_params.p.random_below_ref(&mut rand))
                    })
                    .take((c.slot_per_round * size).next_power_of_two())
                    .collect(),
                    b: std::iter::repeat_with(|| {
                        Integer::from(c.base_params.p.random_below_ref(&mut rand))
                    })
                    .take((c.slot_per_round * size).next_power_of_two())
                    .collect(),
                    p: c.bulk_params.p.clone(),
                    w: c.bulk_params.ring_v.order.clone(),
                    order: c.bulk_params.q.clone(),
                });
            })
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark_compute_prf_bulk, criterion_benchmark_compute_prf_base);
criterion_main!(benches);
