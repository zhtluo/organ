mkdir -p ./log

while IFS= read -r line; do
  IPS+=($line)
done < $1

bash ./script/get_pvt_ip.sh $1 > ./log/pvt.txt

while IFS= read -r line; do
  PIPS+=($line)
done < ./log/pvt.txt

UIPS=($(echo "${IPS[@]}" | tr ' ' '\n' | sort -u | tr '\n' ' '))

echo $UIPS > ./log/uip.txt

# Install the program
for ((i = 0; i <= $(expr ${#UIPS[@]} - 1); i++))
  scp -i ~/organ.pem script-ubuntu/sshd_config ubuntu@${UIPS[$i]}:~/sshd_config &
  ssh -t -i ~/organ.pem ubuntu@${UIPS[$i]} 'bash -ls' < script-ubuntu/aws.sh &
done

wait

# Generate the shares
for d in ./script/config/*; do
  mkdir -p ./log/$d
  RUST_LOG=info ./target/release/organ config ./script/config/$d/oprf58.json ./log/$d &
done

wait

# Upload the shares
for ((i = 0; i <= $(expr ${#UIPS[@]} - 1); i++))
  for d in ./script/config/*; do
    ssh -t -i ~/organ.pem ubuntu@${UIPS[$i]} "mkdir -p ~/organ/log/$d" &
  done
done

wait

for d in ./script/config/*; do
  scp -i ~/organ.pem ./log/$d/bits_32_relay.txt ubuntu@${IPS[0]}:~/organ/log/$d/ &
  scp -i ~/organ.pem ./log/$d/bits_226_relay.txt ubuntu@${IPS[0]}:~/organ/log/$d/ &
  scp -i ~/organ.pem ./log/$d/*.json ubuntu@${IPS[0]}:~/organ/log/$d/ &
  for ((i = 1; i <= $(expr ${#IPS[@]} - 1); i++))
  do
    scp -i ~/organ.pem ./log/$d/bits_32_nid_$(expr $i - 1).txt ubuntu@${IPS[$i]}:~/organ/log/$d/ &
    scp -i ~/organ.pem ./log/$d/bits_226_nid_$(expr $i - 1).txt ubuntu@${IPS[$i]}:~/organ/log/$d/ &
    scp -i ~/organ.pem ./log/$d/*.json ubuntu@${IPS[$i]}:~/organ/log/$d/ &
  done
done

wait
