set -e
mkdir -p ./log/$2
RUST_LOG=info ./target/release/organ config $2 ./log/$2/
RUST_LOG=info ./target/release/organ server 0 $2 ./log/$2/bits_64_relay.txt ./log/$2/bits_226_relay.txt  2> >(tee -a ./log/$2/relay.log >&2) &
sleep 1
for ((i = 0; i < $1; i++))
do
	RUST_LOG=info ./target/release/organ client $i $2 ./log/$2/bits_64_nid_$i.txt ./log/$2/bits_226_nid_$i.txt 2> ./log/$2/client_$i.log &
done

wait
