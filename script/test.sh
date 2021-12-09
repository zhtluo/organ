RUST_LOG=debug ./target/release/organ server 0 ./target/PREPROCESSED_PRF/n10/config.json ./target/PREPROCESSED_PRF/n10/RELAY/bits_32.txt ./target/PREPROCESSED_PRF/n10/RELAY/bits_256.txt &
sleep 1
for i in {0..9}
do
	RUST_LOG=debug ./target/release/organ client $i ./target/PREPROCESSED_PRF/n10/config.json ./target/PREPROCESSED_PRF/n10/bits_32_nid_$i.txt ./target/PREPROCESSED_PRF/n10/bits_256_nid_$i.txt &
done

wait
