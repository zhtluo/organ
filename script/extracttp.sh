cd ./log/50
for d in t*.json; do 
  cd ../../
  echo $d
  for e in 50 100 150 200; do
    python3 ./script/extracttp.py ./log/$e/$d/client_0.log
  done
  cd ./log/50
  echo
done
cd ../../
