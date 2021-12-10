while IFS= read -r line; do
  IPS+=($line)
done < $1

for ip in "${IPS[@]}"
do
    bash ./script-ubuntu/install_node.sh $ip &
done

wait