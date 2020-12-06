FROM i386/ubuntu

MAINTAINER christopher@lord.ac

WORKDIR /root

RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y apt-utils \
 && DEBIAN_FRONTEND=noninteractive dpkg-reconfigure apt-utils \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y \
        automake \
        cmake \
        curl \
        fakeroot \
        g++ \
        git \
        make \
        runit \
        sudo \
        xz-utils

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- -y

ENV PATH=/root/.cargo/bin:$PATH
ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

RUN rustup target add arm-unknown-linux-gnueabihf
COPY config /root/.cargo/config

RUN git clone --depth=1 https://github.com/raspberrypi/tools.git
ENV PATH=/root/tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH

# RUN mkdir restedpi-rust
# COPY . restedpi-rust/
# WORKDIR restedpi-rust

# RUN cargo build --target=arm-unknown-linux-gnueabihf --release

