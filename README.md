# OrgAn

This is a prototype implementation of the OrgAn protocol proposed in the paper 'OrgAn: Organizational Anonymity with Low Latency'. 
The protocol follows a client/relay/server/relay model, where the setup server provides secret shares of a publicly known value to the clients. The clients in the organisation communicate anonymously through the relay with the outside world. The communication proceeds in `Base` and `Bulk` rounds. The clients use `Base` round for slot selection and `Bulk` round to forward their messages in the chosen slots. Each client computes randomness to mask the slot messages as a polynomial ring element using a almost key homomorphic PRF output. 

The relay collects all the messages from all the clients in a Base round, computes the Newton's sum equation system and solves it to obtain a random permutation of client input values. This permutation is used to select slots in the Bulk round. 

The random value chosen for slot selection is `64` bits. For each slot of a particular bulk round, a client can forward  `226` bits of message.  



## Local setup and test

- Make sure that you have Rust installed. (`https://www.rust-lang.org/`)

- Run `cargo build --release` to build the prototype.

- Use `./script_local/test.sh` to start a test run. The config file is located at `./script_local/config.json`.

This will generate the secrets as one guard server, and then launch the specified number of nodes (1 relay + 5 clients by default) to simulate the exchange of the base round and the bulk round under different settings locally to measure the performance. The settings can be checked under `./script_local/config`, and the log will be dumped to `./log/local`.

## Configuration
The network configuration is specified under `./script_local/config`. The number of slots 

- You may analyze the log anyway you want. For simplicity a code snippet is provided under `./script_local/extract.sh`.

Run this script will yield a result like:

```
Optimal round trip time in seconds (base round and bulk round respectively):
onoprf58.json
0.01 0.066

oprf1024.json
0.014 0.015

oprf58.json
0.015 0.014

Round trip time in seconds (base round and bulk round respectively):
rnoprf58.json
0.035 0.125

rprf1024.json
0.014 0.018

rprf58.json
0.012 0.01

Average round time in seconds (both base and bulk round):
tnoprf1024.json
0.26

tprf1024.json
0.031
```

The name of the config file explains the actual setting the experiment is performed under. `prf` means PRF is precomputed and `noprf` means that PRF is computed on-demand. `58` and `1024` describes that amount of bytes each node sends per bulk round.

## Steps to repeat the benchmarks reported

- Set up VPSs and dump their IP into a `txt` file, one per line with the first as relay.

- The script uses `~/organ.pem` as the SSH keypair. Modify if you want to use something else.

- Run `./script/setup.sh <Your IP file>` to build the prototype on each of the machine. Note you may have to modify `./script/get_pvt_ip.sh` to recognize your subnet if your private network address is different from `172.31.*.*`.

- Run `./script/run.sh <Your IP file>` to run all the tests and fetch the log under `./log/`.

- You may analyze the log anyway you want. For simplicity a code snippet is provided under `./script/extract.sh`.
