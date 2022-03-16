script="sleep 10; rm -f ./output_$4.log; RUST_LOG=info ./organ/target/release/organ client $4 ./organ/script/config/$3 ./organ/log/$2/bits_64_nid_$4.txt ./organ/log/$2/bits_226_nid_$4.txt 2> output_$4.log"
ssh -i ~/organ.pem ubuntu@$1 $script
echo ../../log/$3/
mkdir -p ../../log/$3/
scp -i ~/organ.pem ubuntu@$1:./output_$4.log ../../log/$3/client_$4.log
