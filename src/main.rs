#[macro_use]
extern crate log;

use regex::Regex;
use rug::Integer;
use std::env;
use std::fs;
mod client;
mod config;
mod message;
mod server;

fn load_prf(input: &str) -> Vec<Integer> {
    let mut prf = Vec::<Integer>::new();
    for mat in Regex::new(r"[0-9]+").unwrap().find_iter(input) {
        prf.push(Integer::from_str_radix(mat.as_str(), 10).unwrap());
    }
    prf
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!(r"Usage: organ (client|server) <id> <base_prf_file> <bulk_prf_file>");
        return;
    }
    env_logger::init();
    info!("Starting up...");
    debug!("args are {:?}.", args);
    let conf = config::load_config().unwrap();
    info!("Reading from {}...", args[3]);
    let base_prf = load_prf(&fs::read_to_string(&args[3]).unwrap());
    info!("Reading from {}...", args[4]);
    let bulk_prf = load_prf(&fs::read_to_string(&args[4]).unwrap());
    if args[1] == "client" {
        let nid: usize = args[2].parse().unwrap();
        client::main(conf, nid, base_prf, bulk_prf);
    } else if args[1] == "server" {
        server::main(conf);
    } else {
        panic!("Unknown node type. Must be client or server.");
    }
}
