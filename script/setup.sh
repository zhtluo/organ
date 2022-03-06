mkdir -p ./log

while IFS= read -r line; do
  IPS+=($line)
done < $1

UIPS=($(echo "${IPS[@]}" | tr ' ' '\n' | sort -u | tr '\n' ' '))

echo ${UIPS[*]} > ./log/uip.txt

# bash ./script/get_pvt_ip.sh $1 > ./log/pvt.txt

while IFS= read -r line; do
  PIPS+=($line)
done < ./log/pvt.txt


: '
# Install the program
for ((i = 0; i <= $(expr ${#UIPS[@]} - 1); i++)); do
  scp -i ~/organ.pem script/sshd_config ubuntu@${UIPS[$i]}:~/sshd_config &
  ssh -t -i ~/organ.pem ubuntu@${UIPS[$i]} 'bash -ls' < script/aws.sh &
done

wait

# Generate the shares
cd ./script/config/
for d in *; do
  mkdir -p ../../log/$d
  RUST_LOG=info ../../target/release/organ config ./$d/oprf58.json ../../log/$d &
done
cd ../../

wait

'

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
  scp -i ~/organ.pem ../../log/$d/bits_32_relay.txt ubuntu@${IPS[0]}:~/organ/log/$d/ &
  scp -i ~/organ.pem ../../log/$d/bits_226_relay.txt ubuntu@${IPS[0]}:~/organ/log/$d/ &
  scp -i ~/organ.pem ./$d/*.json ubuntu@${IPS[0]}:~/organ/log/$d/ &
  for ((i = 1; i <= $d; i++))
  do
    scp -i ~/organ.pem ../../log/$d/bits_32_nid_$(expr $i - 1).txt ubuntu@${IPS[$i]}:~/organ/log/$d/ &
    scp -i ~/organ.pem ../../log/$d/bits_226_nid_$(expr $i - 1).txt ubuntu@${IPS[$i]}:~/organ/log/$d/ &
    scp -i ~/organ.pem ./$d/*.json ubuntu@${IPS[$i]}:~/organ/log/$d/ &
  done
done
cd ../../

wait
