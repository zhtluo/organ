mkdir -p ./log

while IFS= read -r line; do
  IPS+=($line)
done < $1

UIPS=($(echo "${IPS[@]}" | tr ' ' '\n' | sort -u | tr '\n' ' '))

echo ${IPS[*]}

echo ${UIPS[*]} > ./log/uip.txt

bash ./script/get_pvt_ip.sh $1 > ./log/pvt.txt

while IFS= read -r line; do
  PIPS+=($line)
done < ./log/pvt.txt

# Correct the config
cd ./script/config/
for d in *; do
  for c in $d/*; do
    sed -i 's/[0-9]\{1,3\}\.[0-9]\{1,3\}\.[0-9]\{1,3\}\.[0-9]\{1,3\}'/${PIPS[0]}/ $c 
  done
done

# Install the program
for ((i = 0; i <= $(expr ${#UIPS[@]} - 1); i++)); do
  # Upload new SSH config. Enable if needed.
  # scp -i ~/organ.pem script/sshd_config ubuntu@${UIPS[$i]}:~/sshd_config &
  ssh -t -i ~/organ.pem ubuntu@${UIPS[$i]} 'bash -ls' < script/aws.sh &
done

wait

# Upload the config
cd ./script/config/
for d in *; do
  for ((i = 0; i <= $d; i++))
  do
    scp -r -i ~/organ.pem ../config ubuntu@${IPS[$i]}:~/organ/script/ &
    scp -r -i ~/organ.pem ../config ubuntu@${IPS[$i]}:~/organ/script/ &
  done
done

# Generate the shares
cd ./script/config/
for d in *; do
  mkdir -p ../../log/$d
  RUST_LOG=info ../../target/release/organ config ./$d/oprf58.json ../../log/$d &
done
cd ../../

wait

cd ./script/config/
# Upload the shares
for ((i = 0; i <= $(expr ${#UIPS[@]} - 1); i++)); do
  for d in *; do
    ssh -t -i ~/organ.pem ubuntu@${UIPS[$i]} "mkdir -p ~/organ/log/$d" &
  done
done
cd ../../

wait

cd ./script/config/
for d in *; do
  scp -i ~/organ.pem ../../log/$d/bits_64_relay.txt ubuntu@${IPS[0]}:~/organ/log/$d/ &
  scp -i ~/organ.pem ../../log/$d/bits_226_relay.txt ubuntu@${IPS[0]}:~/organ/log/$d/ &
  for ((i = 1; i <= $d; i++))
  do
    scp -i ~/organ.pem ../../log/$d/bits_64_nid_$(expr $i - 1).txt ubuntu@${IPS[$i]}:~/organ/log/$d/ &
    scp -i ~/organ.pem ../../log/$d/bits_226_nid_$(expr $i - 1).txt ubuntu@${IPS[$i]}:~/organ/log/$d/ &
  done
done
cd ../../

wait

