cd ./log/200
for d in o*.json; do 
  cd ../../
  echo $d
  for e in 50 100 150 200; do
    python3 ./script/extractp.py ./log/$e/$d/client_0.log
  done
  cd ./log/200
  echo
done
for d in r*.json; do 
  cd ../../
  echo $d
  for e in 50 100 150 200; do
    python3 ./script/extractp.py ./log/$e/$d/client_0.log
  done
  cd ./log/200
  echo
done
cd ../../
