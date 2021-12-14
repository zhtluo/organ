#[macro_use]
extern crate log;

use regex::Regex;
use rug::Integer;
use std::env;
use std::fs;
mod client;
mod config;
mod flint;
mod message;
mod net;
mod server;
mod timing;

fn load_prf(input: &str) -> Vec<Integer> {
    let mut prf = Vec::<Integer>::new();
    for mat in Regex::new(r"[0-9]+").unwrap().find_iter(input) {
        prf.push(Integer::from_str_radix(mat.as_str(), 10).unwrap());
    }
    prf
}

#[async_std::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        println!(
            r"Usage: organ (client|server) <id> <config_file> <base_prf_file> <bulk_prf_file>"
        );
        return;
    }
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();
    info!("Starting up...");
    debug!("args: {:?}", args);
    info!("Reading from {}...", args[3]);
    let conf = config::load_config(&args[3]).unwrap();
    info!("Reading from {}...", args[4]);
    let base_prf = load_prf(&fs::read_to_string(&args[4]).unwrap());
    info!("Reading from {}...", args[5]);
    let bulk_prf = load_prf(&fs::read_to_string(&args[5]).unwrap());
    if args[1] == "client" {
        let nid: usize = args[2].parse().unwrap();
        client::main_prifi(conf, nid, base_prf, bulk_prf);
    } else if args[1] == "server" {
        server::main_prifi(conf, base_prf, bulk_prf).await;
    } else {
        panic!("Unknown node type. Must be client or server.");
    }
}
