IPS=()

while IFS= read -r line; do
  IPS+=($line)
done < $1

for ip in "${IPS[@]}"
do
    ssh -i ~/organ.pem ubuntu@$ip 'ip address show' | \
    grep "inet .* brd" | \
    sed 's/ brd.*//g' | \
    sed 's/inet //' | \
    sed 's;/.*;;g' | \
    sed 's/.* //g'
done
