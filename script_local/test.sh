cd "${0%/*}"
cd ..
mkdir -p log/local

# Generate the shares
cd ./script_local/config/
for d in *; do
  mkdir -p ../../log/local/$d
  RUST_LOG=info ../../target/release/organ config ./$d/oprf58.json ../../log/local/$d &
done
cd ../../

wait

killall organ

# Run the tests locally
cd ./script_local/config/
for d in *; do
  for c in $d/*; do
	mkdir -p ../../log/local/$c
	# Launch the server
	RUST_LOG=INFO ../../target/release/organ server 0 $c ../../log/local/$d/bits_32_relay.txt \
	../../log/local/$d/bits_226_relay.txt 2> >(tee -a ../../log/local/$c/relay.log >&2) &
	sleep 1
	# Launch the clients
    for ((i = 0; i < $d; i++)); do
	  RUST_LOG=INFO ../../target/release/organ client $i $c ../../log/local/$d/bits_32_nid_$i.txt \
	  ../../log/local/$d/bits_226_nid_$i.txt 2> ../../log/local/$c/client_$i.log &
    done
    wait
  done
done
cd ../../

wait
