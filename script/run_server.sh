script="RUST_LOG=debug ./organ server 0 ./config.json ./base.txt ./bulk.txt 2> >(tee -a output.log >&2)"
ssh arch@$1 $script
scp arch@$1:./output.log ./log/server.log
