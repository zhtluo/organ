while IFS= read -r line; do
  IPS+=($line)
done < $1

bash ./script/run_server.sh ${IPS[0]} &
for ((i = 1; i <= $(expr ${#IPS[@]} - 1); i++))
do
    bash ./script/run_client.sh ${IPS[$i]} $2 $(expr $i - 1) &
done

wait

for ip in "${IPS[@]}"
do
    ssh arch@$ip "killall organ" &
done

wait