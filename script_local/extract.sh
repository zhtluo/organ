cd "${0%/*}"
cd ..
cd ./log/local/5
echo "Optimal round trip time (in seconds, base round and bulk round respectively):"
for d in o*.json; do 
  cd ../../../
  echo $d
  for e in 5; do
    python3 ./script/extract.py ./log/local/$e/$d/client_0.log
  done
  cd ./log/local/5
  echo
done
echo "Round trip time (in seconds, base round and bulk round respectively):"
for d in r*.json; do 
  cd ../../../
  echo $d
  for e in 5; do
    python3 ./script/extract.py ./log/local/$e/$d/client_0.log
  done
  cd ./log/local/5
  echo
done
echo "Average round time (in seconds, both base and bulk round):"
for d in a*.json; do 
  cd ../../../
  echo $d
  for e in 5; do
    python3 ./script/extractt.py ./log/local/$e/$d/client_0.log
  done
  cd ./log/local/5
  echo
done
cd ../../../
