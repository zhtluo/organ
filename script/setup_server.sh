ssh -t arch@$1 'bash -ls' < script/aws.sh
scp ./target/release/organ arch@$1:./
scp ./$2/config.json arch@$1:./
scp ./$2/bits_32.txt arch@$1:./base.txt
scp ./$2/bits_226.txt arch@$1:./bulk.txt
