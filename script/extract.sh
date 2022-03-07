cd ./log/50
for d in r*.json; do 
  cd ../../
  for e in 50 100 150 200; do
    python3 ./script/extract.py ./log/$e/$d/client_0.log
  done
  cd ./log/50
  echo
done
cd ../../
