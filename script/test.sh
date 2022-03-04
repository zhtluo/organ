set -e
mkdir -p ./log/$2
RUST_LOG=debug ./target/release/organ config $2 ./log/$2/
RUST_LOG=debug ./target/release/organ server 0 $2 ./log/$2/bits_32_relay.txt ./log/$2/bits_226_relay.txt &
sleep 1
for ((i = 0; i < $1; i++))
do
	RUST_LOG=debug ./target/release/organ client $i $2 ./log/$2/bits_32_nid_$i.txt ./log/$2/bits_226_nid_$i.txt &
done

wait
