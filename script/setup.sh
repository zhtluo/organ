mkdir -p ./log

while IFS= read -r line; do
  IPS+=($line)
done < $1

bash ./script/get_pvt_ip.sh $1 > ./log/pvt.txt

while IFS= read -r line; do
  PIPS+=($line)
done < ./log/pvt.txt

echo '{' > $2/config.json
echo -n '"server_addr": "' >> $2/config.json
echo -n ${PIPS[0]} >> $2/config.json
echo ':7000",' >> $2/config.json
echo -n '"client_size": ' >> $2/config.json
echo -n $(expr ${#PIPS[@]} - 1) >> $2/config.json
echo ',' >> $2/config.json
echo '"round": 100' >> $2/config.json
echo "}" >> $2/config.json

bash ./script/setup_server.sh ${IPS[0]} $2 &
for ((i = 1; i <= $(expr ${#IPS[@]} - 1); i++))
do
    bash ./script/setup_server.sh ${IPS[$i]} $2 $(expr $i - 1) &
done

wait
