#[macro_use]
extern crate log;

use organ::{client, config, guard, server};
use regex::Regex;
use rug::Integer;
use std::env;
use std::fs;

fn load_prf(input: &str) -> Vec<Integer> {
    let mut prf = Vec::<Integer>::new();
    for mat in Regex::new(r"[0-9]+").unwrap().find_iter(input) {
        prf.push(Integer::from_str_radix(mat.as_str(), 10).unwrap());
    }
    prf
}

fn generate_prf(client_size: usize, params: &config::ProtocolParams) {
    let shares: Vec<Vec<Integer>> = (0..params.vector_len)
        .map(|_| guard::generate_sum_shares(client_size, &params.ring_v.order, &Integer::from(1)))
        .collect();
    let shares: Vec<Vec<Integer>> = (0..client_size)
        .map(|i| shares.iter().map(|v| v[i].clone()).collect())
        .collect();
    for i in 0..client_size {
        std::fs::write(
            format!("bits_{}_nid_{}.txt", params.bits.to_string(), i.to_string()),
            format!("{:?}", guard::message_gen(&params, shares[i].clone())),
        )
        .unwrap();
    }
    std::fs::write(
        format!("bits_{}_relay.txt", params.bits.to_string()),
        format!(
            "{:?}",
            guard::message_gen(&params, vec![Integer::from(1); params.vector_len])
        ),
    )
    .unwrap();
}

#[async_std::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();
    info!("Starting up...");
    debug!("args: {:?}", args);
    if args[1] == "config" {
        info!("Reading from {}...", args[2]);
        let conf = config::load_config(&args[2]).unwrap();
        generate_prf(conf.client_size, &conf.base_params);
        generate_prf(conf.client_size, &conf.bulk_params);
    } else if args[1] == "client" || args[1] == "server" {
        info!("Reading from {}...", args[3]);
        let conf = config::load_config(&args[3]).unwrap();
        info!("Reading from {}...", args[4]);
        let base_prf = load_prf(&fs::read_to_string(&args[4]).unwrap());
        info!("Reading from {}...", args[5]);
        let bulk_prf = load_prf(&fs::read_to_string(&args[5]).unwrap());
        if args[1] == "client" {
            let nid: usize = args[2].parse().unwrap();
            client::main(conf, nid, base_prf, bulk_prf);
        } else if args[1] == "server" {
            server::main(conf, base_prf, bulk_prf).await;
        }
    } else {
        println!(r"Usage:");
        println!(r"organ (client|server) <id> <config_file> <base_prf_file> <bulk_prf_file>");
        println!(r"organ config <config_file>");
        return;
    }
}
