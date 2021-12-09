script="sleep 10; RUST_LOG=debug ./organ client $2 ./config.json ./base.txt ./bulk.txt 2> output.log"
ssh arch@$1 $script
scp arch@$1:./output.log ./log/$2.log
