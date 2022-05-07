# syntax=docker/dockerfile:1

FROM ubuntu:bionic
SHELL ["/bin/bash", "-c"]
RUN apt update
RUN apt install -y git wget build-essential libgmp-dev libmpfr-dev m4
RUN wget https://www.flintlib.org/flint-2.8.4.tar.gz
RUN tar -xf flint-2.8.4.tar.gz
WORKDIR /flint-2.8.4
RUN ./configure
RUN make install
RUN ldconfig
WORKDIR /
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.60.0 \
    rustArch=x86_64-unknown-linux-gnu
RUN set -eux; \
    url="https://static.rust-lang.org/rustup/archive/1.24.3/${rustArch}/rustup-init"; \
    wget "$url"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;
RUN git clone https://github.com/zhtluo/organ.git
WORKDIR /organ
RUN cargo build --release
CMD /bin/bash
