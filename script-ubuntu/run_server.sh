script="rm -f ./output.log; RUST_LOG=info ./organ server 0 ./config.json ./base.txt ./bulk.txt 2> >(tee -a output.log >&2)"
ssh -i ~/organ.pem ubuntu@$1 $script
scp -i ~/organ.pem ubuntu@$1:./output.log ./log/output.log
