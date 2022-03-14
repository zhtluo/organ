# organ

This is a prototype implementation for the paper "OrgAn: Organizational Anonymity with Low Latency".

## Steps to run a local setup and test

- Make sure that you have Rust installed.

- Run `cargo build --release` to build the prototype.

- Use `./script_local/test.sh` to start a test run. The config file is located at `./script_local/config.json`.

## Steps to repeat the experiment

- Set up VPSs and dump their IP into a `txt` file, one per line with the first as relay.

- Run `./script/setup.sh <Your IP file>` to build the prototype on each of the machine. Note you may have to modify `./script/get_pvt_ip.sh` to recognize your subnet if your private network address is different from `172.31.*.*`.

- Run `./script/run.sh <Your IP file>` to run all the tests and fetch the log under `./log/`.

- You may analyze the log anyway you want. For simplicity a code snippet is provided under `./script/extract.sh`.