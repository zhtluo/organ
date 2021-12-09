script="sleep 10; RUST_LOG=debug ./organ client $2 ./config.json ./base_$2.txt ./bulk_$2.txt 2> output_$2.log"
ssh arch@$1 $script
scp arch@$1:./output_$2.log ./log/output_$2.log
