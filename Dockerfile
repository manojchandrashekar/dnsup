FROM ubuntu:18.04
USER root

RUN mkdir /opt/dnsup
WORKDIR /opt/dnsup

RUN apt-get update -y
RUN apt-get upgrade -y
RUN apt-get install -y locales \
    && localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8
ENV LANG en_US.utf8
# RUN apt-get install -y gcc-mingw-w64-x86-x86_64
RUN apt-get install -y g++-mingw-w64-x86-64
RUN apt-get install -y wine64
RUN apt-get install -y libssl-dev
RUN apt-get install -y pkg-config
RUN apt-get install -y build-essential
RUN apt-get install -y gcc
RUN apt-get install -y python3-dev
RUN apt-get install -y software-properties-common
RUN apt-get install -y make
RUN apt-get install -y libffi-dev
RUN apt-get install -y libtool
RUN apt-get install -y gcc-x86-64-linux-gnu
RUN apt-get install -y libudev-dev
RUN apt-get install -y curl
RUN apt-get install -y wget
RUN apt-get install -y cmake
RUN apt-get install -y clang
RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:$PATH"

RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add i686-unknown-linux-gnu
RUN rustup target add x86_64-unknown-linux-musl

# RUN apt-get install -y linux-musl-dev
RUN apt-get install -y musl-dev
RUN apt-get install -y musl-tools

# ENV OPENSSL_LIB_DIR="/usr/lib/x86_64-linux-gnu"
# ENV OPENSSL_INCLUDE_DIR="/usr/include/openssl"
# ENV CC=gcc
# ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'

# RUN cargo build --release --locked
# RUN cargo build --release --target aarch64-unknown-linux-gnu --locked
# RUN cargo build --release --target x86_64-pc-windows-gnu --locked
# RUN cargo build --release --target i686-unknown-linux-gnu --locked

CMD ["echo" "Builds complete"]