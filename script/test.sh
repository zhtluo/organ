RUST_LOG=info ./target/debug/organ server 0 ./target/PREPROCESSED_PRF/n$1/config.json ./target/PREPROCESSED_PRF/n$1/RELAY/bits_32.txt ./target/PREPROCESSED_PRF/n$1/RELAY/bits_226.txt &
sleep 1
for ((i = 0; i < $1; i++))
do
	RUST_LOG=info ./target/debug/organ client $i ./target/PREPROCESSED_PRF/n$1/config.json ./target/PREPROCESSED_PRF/n$1/bits_32_nid_$i.txt ./target/PREPROCESSED_PRF/n$1/bits_226_nid_$i.txt &
done

wait
