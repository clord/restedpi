FROM ubuntu

MAINTAINER christopher@lord.ac

WORKDIR /root

RUN dpkg --add-architecture i386

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

RUN git clone --depth=1 https://github.com/raspberrypi/tools.git

RUN DEBIAN_FRONTEND=noninteractive sudo apt-get install -y \
                    libc6:i386 libncurses5:i386 libstdc++6:i386 \
                    lib32z1 gcc-multilib 

RUN DEBIAN_FRONTEND=noninteractive sudo apt-get install -y \
                    libc6-armel-cross libc6-dev-armel-cross \
                    binutils-arm-linux-gnueabi libncurses5-dev build-essential \
                    bison flex libssl-dev bc

RUN curl https://sh.rustup.rs -o rustup.sh && \
    sh rustup.sh --default-host i686-unknown-linux-gnu --default-toolchain nightly-2020-10-07 -y

ENV PATH=/root/.cargo/bin:/root/tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH
ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

RUN rustup target add arm-unknown-linux-gnueabihf
COPY config /root/.cargo/config

ENV PATH=/root/tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH

RUN mkdir restedpi-rust
# WORKDIR restedpi-rust
# RUN cargo build --target=arm-unknown-linux-gnueabihf --release

COPY sqlite-autoconf-3360000 sqlite
WORKDIR sqlite
RUN ./configure --prefix="/root/tools/arm-bcm2708/arm-linux-gnueabihf" --host=arm-linux-gnueabihf
RUN make && make install

