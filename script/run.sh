while IFS= read -r line; do
  IPS+=($line)
done < $1

for ip in "${IPS[@]}"
do
    ssh -i ~/organ.pem ubuntu@$ip "killall organ" &
done

for d in ./script/config/*; do
  for c in ./script/config/$d/*; do
    bash ./script-ubuntu/run_server.sh ${IPS[0]} $d $c &
    for ((i = 1; i <= $(expr ${#IPS[@]} - 1); i++)); do
      bash ./script-ubuntu/run_client.sh ${IPS[$i]} $d $c $(expr $i - 1) &
    done
    wait
  done
done

wait
