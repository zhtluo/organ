ssh -t arch@$1 'bash -ls' < script/aws.sh
scp ./target/release/organ arch@$1:./
scp ./$2/config.json arch@$1:./
scp ./$2/bits_32_nid_$3.txt arch@$1:./base_$3.txt
scp ./$2/bits_226_nid_$3.txt arch@$1:./bulk_$3.txt
