RUST_LOG=info ./target/debug/organ server 0 ./target/PREPROCESSED_PRF_2node/n2/RELAY/bits_32.txt ./target/PREPROCESSED_PRF_2node/n2/RELAY/bits_226.txt &
sleep 1
RUST_LOG=info ./target/debug/organ client 0 ./target/PREPROCESSED_PRF_2node/n2/bits_32_nid_0.txt ./target/PREPROCESSED_PRF_2node/n2/bits_226_nid_0.txt &
RUST_LOG=info ./target/debug/organ client 1 ./target/PREPROCESSED_PRF_2node/n2/bits_32_nid_1.txt ./target/PREPROCESSED_PRF_2node/n2/bits_226_nid_1.txt &
