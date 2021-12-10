script="sleep 10; rm -f ./output_$2.log; RUST_LOG=info ./organ client $2 ./config.json ./base_$2.txt ./bulk_$2.txt 2> output_$2.log"
ssh -i ~/organ.pem ubuntu@$1 $script
scp -i ~/organ.pem ubuntu@$1:./output_$2.log ./log/output_$2.log
