script="rm -f ./output.log; RUST_LOG=info ./organ/target/release/organ prifi server 0 ./organ/script/prifi/$3 2> >(tee -a output.log >&2)"
ssh -i ~/organ.pem ubuntu@$1 $script
mkdir -p ../../log/$3/
scp -i ~/organ.pem ubuntu@$1:./output.log ../../log/$3/relay.log
