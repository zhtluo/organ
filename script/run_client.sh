script="sleep 10; rm -f ./output_$2.log; RUST_LOG=info ./organ/target/release/organ client $4 ./organ/script/config/$2/$3.json ./organ/log/$2/bits_32_relay.txt ./organ/log/$2/bits_226_relay.txt 2> output_$2.log"
ssh -i ~/organ.pem ubuntu@$1 $script
mkdir -p ../../log/$2/$3/
scp -i ~/organ.pem ubuntu@$1:./output_$2.log ../../log/$2/$3/client_$4.log
