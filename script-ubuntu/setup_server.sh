scp -i ~/organ.pem ./$2/config.json ubuntu@$1:./
scp -i ~/organ.pem ./$2/RELAY/bits_32.txt ubuntu@$1:./base.txt
scp -i ~/organ.pem ./$2/RELAY/bits_256.txt ubuntu@$1:./bulk.txt
