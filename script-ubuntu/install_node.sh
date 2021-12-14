scp -i ~/organ.pem script-ubuntu/sshd_config ubuntu@$1:~/sshd_config
ssh -t -i ~/organ.pem ubuntu@$1 'bash -ls' < script-ubuntu/aws.sh
