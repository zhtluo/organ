while IFS= read -r line; do
  IPS+=($line)
done < $1

for ip in "${IPS[@]}"
do
    ssh -i ~/organ.pem ubuntu@$ip "killall organ" &
done

wait

cd ./script/prifi/
for d in *; do
  for c in $d/*; do
    bash ../run_server_prifi.sh ${IPS[0]} $d $c &
    for ((i = 1; i <= $d; i++)); do
      bash ../run_client_prifi.sh ${IPS[$i]} $d $c $(expr $i - 1) &
    done
    wait
  done
done
cd ../../

wait
