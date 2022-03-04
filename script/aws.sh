rm -rf ./organ
git clone https://github.com/zhtluo/organ.git
cd organ
git pull

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install-rust.sh
bash install-rust.sh -y
source $HOME/.cargo/env
cargo build --release

sudo mv /home/ubuntu/sshd_config /etc/ssh/sshd_config
sudo systemctl restart ssh
