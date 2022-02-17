#[macro_use]
extern crate log;

use organ::{client, config, guard, server};
use rug::Integer;
use std::env;
use std::fs;

fn load_prf(input: &Vec<u8>) -> guard::Setup {
    bincode::deserialize::<guard::Setup>(input).unwrap()
}

fn generate_prf(client_size: usize, params: &config::ProtocolParams) {
    let shares: Vec<Vec<Integer>> = (0..params.vector_len)
        .map(|_| guard::generate_sum_shares(client_size, &params.ring_v.order, &Integer::from(1)))
        .collect();
    let shares: Vec<Vec<Integer>> = (0..client_size)
        .map(|i| shares.iter().map(|v| v[i].clone()).collect())
        .collect();
    let setup_values: Vec<guard::SetupValues> = (0..client_size)
        .map(|i| guard::gen_setup_values(&params, &shares[i]))
        .collect();
    for i in 0..client_size {
        std::fs::write(
            format!("bits_{}_nid_{}.txt", params.bits.to_string(), i.to_string()),
            bincode::serialize(&guard::Setup::SetupValues(setup_values[i].clone())).unwrap(),
        )
        .unwrap();
    }
    std::fs::write(
        format!("bits_{}_relay.txt", params.bits.to_string()),
        bincode::serialize(&guard::Setup::SetupRelay(guard::gen_setup_relay(
            &params,
            &setup_values,
        )))
        .unwrap(),
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
        let base_prf = load_prf(&fs::read(&args[4]).unwrap());
        info!("Reading from {}...", args[5]);
        let bulk_prf = load_prf(&fs::read(&args[5]).unwrap());
        if args[1] == "client" {
            let nid: usize = args[2].parse().unwrap();
            if let guard::Setup::SetupValues(base) = base_prf {
                if let guard::Setup::SetupValues(bulk) = bulk_prf {
                    client::main(conf, nid, base, bulk);
                }
            }
        } else if args[1] == "server" {
            if let guard::Setup::SetupRelay(base) = base_prf {
                if let guard::Setup::SetupRelay(bulk) = bulk_prf {
                    server::main(conf, base, bulk).await;
                }
            }
        }
    } else {
        println!(r"Usage:");
        println!(r"organ (client|server) <id> <config_file> <base_prf_file> <bulk_prf_file>");
        println!(r"organ config <config_file>");
        return;
    }
}
