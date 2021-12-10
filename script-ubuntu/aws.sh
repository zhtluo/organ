mkdir -p organ-git
cd organ-git
git clone https://github.com/zhtluo/organ.git
cd organ

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install-rust.sh
bash install-rust.sh -y
source $HOME/.cargo/env
cargo build --release

cp ./target/release/organ ../../
