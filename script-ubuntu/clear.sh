while IFS= read -r line; do
  IPS+=($line)
done < $1

for ip in "${IPS[@]}"
do
    ssh -i ~/organ.pem ubuntu@$ip "killall organ" &
done

wait