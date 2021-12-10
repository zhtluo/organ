scp -i ~/organ.pem ./$2/config.json ubuntu@$1:./
scp -i ~/organ.pem ./$2/bits_32_nid_$3.txt ubuntu@$1:./base_$3.txt
scp -i ~/organ.pem ./$2/bits_256_nid_$3.txt ubuntu@$1:./bulk_$3.txt
