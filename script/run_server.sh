script="rm -f ./output.log; RUST_LOG=info ./organ/target/release/organ server 0 ./organ/script/config/$3 ./organ/log/$2/bits_64_relay.txt ./organ/log/$2/bits_226_relay.txt 2> >(tee -a output.log >&2)"
ssh -i ~/organ.pem ubuntu@$1 $script
mkdir -p ../../log/$3/
scp -i ~/organ.pem ubuntu@$1:./output.log ../../log/$3/relay.log
