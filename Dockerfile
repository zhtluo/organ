FROM ubuntu:bionic
RUN apt update
RUN apt install -y build-essential libgmp-dev libmpfr-dev m4
RUN wget https://www.flintlib.org/flint-2.8.4.tar.gz
RUN tar -xf flint-2.8.4.tar.gz
RUN cd flint-2.8.4/
RUN ./configure
RUN sudo make install
RUN sudo ldconfig
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
RUN source $HOME/.cargo/env
RUN git clone https://github.com/zhtluo/organ.git
WORKDIR /organ
RUN cargo build --release
CMD bash ./script_local/test.sh
