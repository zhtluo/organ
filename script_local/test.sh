set -e
cd "${0%/*}"
cd ..
mkdir -p log
RUST_LOG=debug ./target/release/organ config ./script_local/config.json ./log
RUST_LOG=debug ./target/release/organ server 0 ./script_local/config.json ./log/bits_32_relay.txt ./log/bits_226_relay.txt &
sleep 1
for ((i = 0; i < 5; i++))
do
	RUST_LOG=debug ./target/release/organ client $i ./script_local/config.json ./log/bits_32_nid_$i.txt ./log/bits_226_nid_$i.txt &
done

wait
