set -e
cd ./target/n$1/
RUST_LOG=debug ../release/organ config ./config.json
RUST_LOG=debug ../release/organ server 0 ./config.json ./bits_32_relay.txt ./bits_226_relay.txt &
sleep 1
for ((i = 0; i < $1; i++))
do
	RUST_LOG=debug ../release/organ client $i ./config.json ./bits_32_nid_$i.txt ./bits_226_nid_$i.txt &
done

wait
