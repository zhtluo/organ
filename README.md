# OrgAn: Organizational Anonymity with Low Latency

This is a prototype implementation of the OrgAn protocol proposed in the paper 'OrgAn: Organizational Anonymity with Low Latency'. 
The protocol follows a client/relay/server model, where the setup server provides secret shares of a publicly known value to the clients. The clients in the organisation communicate anonymously through the relay with the outside world. The communication proceeds in Base and Bulk rounds. 

The clients use Base round for slot selection and Bulk round to forward their messages in the chosen slots. Each client computes randomness to mask the slot messages as a polynomial ring element using a almost key-homomorphic PRF output. The relay collects all the messages from all the clients in a Base round, computes the Newton's sum equation system and solves it to obtain a random permutation of client input values. This permutation is used to select slots in the Bulk round. Clients choose a 64 bit random value for slot selection in the Base round. In the Bulk round, a client can forward 226 bits of message per allotted slot.  

## Local setup and test

- Make sure that you have Rust installed. (`https://www.rust-lang.org/`)

- Clone the repository using `git clone https://github.com/zhtluo/organ.git`.

- Change directory with `cd organ` and use `cargo build --release` to build the prototype.

- Use `./script_local/test.sh` to start a test-run.

The default local test run launches one setup server which generates client secret shares and outputs them to `./log/local`. Then the specified number of processes (1 relay + 5 clients by default) are launched to simulate the exchange of the base round and the bulk round messages among them. Different configurations for different message lengths and parameters can be used to measure the performance. 

## Configuration and Output Logs

For the local tests, the protocol configuration is specified in `./script_local/config`, and the log, including timestamps on each round, is dumped to `./log/local/<setting name>/`.

- The log may be analyzed in any manner. For simplicity a code snippet is provided under `./script_local/extract.sh`.

Running this script will yield a result like:

```
Optimal round trip time (in seconds, base round and bulk round respectively):
optimal_rtt_no_prf_preprocessing_58Bmessage.json
0.007 0.057

optimal_rtt_with_prf_preprocessing_1024Bmessage.json
0.015 0.015

optimal_rtt_with_prf_preprocessing_58Bmessage.json
0.015 0.014

Round trip time (in seconds, base round and bulk round respectively):
rtt_no_prf_preprocessing_58Bmessage.json
0.054 0.095

rtt_with_prf_preprocessing_1024Bmessage.json
0.018 0.01

rtt_with_prf_preprocessing_58Bmessage.json
0.013 0.01

Average round time (in seconds, both base and bulk round):
avg_rtt_no_prf_preprocessing_1024Bmessage.json
0.291

avg_rtt_with_prf_preprocessing_1024Bmessage.json
0.048
```

For different experiments, we use different configuration files, specified under `script_local/config/#no_of_clients/`
The name of each config file explains the actual setting the experiment is performed under.

## Steps to repeat the benchmarks reported in the paper

- Set up AWS and add the node IPs into a `.txt` file, one per line with the first IP being the IP of the relay node.

- The scripts (eg: `run.sh`, `setup.sh`)from the folder `scripts` use an `~/organ.pem` as the SSH keypair to access the AWS machines. Modify if you want to use something else.

- Run `./script/setup.sh <Your IP file>` to build the prototype on each of the machine. Note you may have to modify `./script/get_pvt_ip.sh` to recognize your subnet if your private network address is different from `172.31.*.*`.

- Run `./script/run.sh <Your IP file>` to run all the tests and fetch the log under `./log/`.

- You may analyze the log anyway you want. For simplicity a code snippet is provided under `./script/extract.sh`.

## Further details on the options available for the protocol configuration. 
The configuration `.json` files of local test `./script_local/config` or AWS network tests in `/script/config/#no_of_clients/` offer the below variables which can be changed from the default values specified (also viewable in `/src/config.rs`)  are stated below. Each test is accompanied with a `.json` file that specifies the setting for that run. An example of the file should look like this:

```
{
  "server_addr": "127.0.0.1:8001",
  "client_size": 5,
  "base_params": {
    "p": {
      "radix": 16,
      "value": "ffffffffffffffc5"
    },
    "q": {
      "radix": 16,
      "value": "fffffffffffffffffffdd"
    },
    "ring_v": {
      "order": {
        "radix": 16,
        "value": "39000000000000000000000001"
      },
      "root": {
        "radix": 16,
        "value": "1e8593afe765eb54ad28c5a71a"
      },
      "scale": {
        "radix": 16,
        "value": "1000000000000000000000000"
      }
    },
    "vector_len": 2048,
    "bits": 64,
    "group_nid": 714
  },
  "bulk_params": {
    "p": {
      "radix": 16,
      "value": "3fffffffffffffffffffffffffffffffffffffffffffffffffffffffb"
    },
    "q": {
      "radix": 16,
      "value": "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141"
    },
    "ring_v": {
      "order": {
        "radix": 16,
        "value": "1c000000000000000000000000000000000000000000000000000000000000000000000001"
      },
      "root": {
        "radix": 10,
        "value": "2187"
      },
      "scale": {
        "radix": 16,
        "value": "4000000000000000000000000000000000000000000000000000000000000000000000000"
      }
    },
    "vector_len": 8192,
    "bits": 226,
    "group_nid": 733
  },
  "round": 10,
  "slot_per_round": 3,
  "do_blame": false,
  "do_unzip": false,
  "do_delay": false,
  "do_ping": false
}
```

1. `server_addr`: The address of the relay and the port used. **Must be included.**
1. `client_size`: The number of clients. **Must be included.**
1. `base_params`: The parameters for the base round, including `p`, `q`, `v`, the length of the vector in the communication `vector_len`, number of bits per round `bits`, and the ECC group id for the blame protocol as specified by OpenSSL `group_nid`. If omitted, the default value will be used.
1. `bulk_params`: The parameters for the bulk round, same as the base round. If omitted, the default value will be used.
1. `round`: The total number of rounds to run. **Must be included.**
1. `slot_per_round`: How many slots does each client use per bulk round. **Must be included.**
1. `do_blame`: Whether or not to test blame protocol by running it every round. Defaults to false.
1. `do_unzip`: Whether or not to unzip and compute PRF values on-demand. Defaults to false.
1. `do_delay`: Whether or not to delay before sending message. Useful in measuring optimal round trip time. Defaults to false.
1. `do_ping`: Whether or not to simulate the real environment by performing a ping after the bulk round. Defaults to false.
