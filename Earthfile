VERSION 0.7
FROM --platform=linux/amd64 ubuntu
WORKDIR /root
ENV DEBIAN_FRONTEND="noninteractive"
ENV PATH=/root/.cargo/bin:/root/tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH
ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

builder:
  RUN dpkg --add-architecture i386

  RUN apt-get update
  RUN apt-get install -y apt-utils && dpkg-reconfigure apt-utils
  RUN apt-get install -y \
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

  GIT CLONE https://github.com/raspberrypi/tools.git /root/tools

  RUN apt-get install -y \
                      libc6:i386 libncurses5:i386 libstdc++6:i386 \
                      lib32z1 gcc-multilib

  RUN apt-get install -y \
                      libc6-armel-cross libc6-dev-armel-cross \
                      binutils-arm-linux-gnueabi libncurses5-dev build-essential \
                      bison flex libssl-dev bc

  RUN apt-get install -y \
                      libc6:i386 libncurses5:i386 libstdc++6:i386 \
                      lib32z1 gcc-multilib

  RUN apt-get install -y \
                      libc6-armel-cross libc6-dev-armel-cross \
                      binutils-arm-linux-gnueabi libncurses5-dev build-essential \
                      bison flex libssl-dev bc


sqlite:
  FROM +builder
  COPY dockerfiles/ci-build/sqlite-autoconf-3360000 sqlite
  WORKDIR /root/sqlite
  RUN ./configure --prefix="/root/sqlite-arm" --host=arm-linux-gnueabihf
  RUN make
  RUN make install
  SAVE ARTIFACT /root/sqlite-arm /sqlite-arm

build:
  FROM +builder

  RUN curl https://sh.rustup.rs -o rustup.sh && \
    sh rustup.sh --default-host i686-unknown-linux-gnu \
        --default-toolchain nightly-2020-10-07 -y

  COPY +sqlite/sqlite-arm /root/sqlite-arm
  COPY dockerfiles/ci-build/config /root/.cargo/config

  RUN rustup target add arm-unknown-linux-gnueabihf

  # Do a cache build
  RUN mkdir /root/src && touch /root/src/lib.rs
  COPY Cargo.toml Cargo.lock .
  RUN RUSTFLAGS='-L /root/sqlite-arm/lib' cargo build --features=raspberrypi --target=arm-unknown-linux-gnueabihf --release
  RUN rm -fr /root/src

  # Do a real build
  COPY build.rs diesel.toml  .
  COPY static ./static
  COPY migrations ./migrations
  COPY src ./src
  RUN RUSTFLAGS='-L /root/sqlite-arm/lib' cargo build --features=raspberrypi --target=arm-unknown-linux-gnueabihf --release
  RUN ls -la target/arm-unknown-linux-gnueabihf
  SAVE ARTIFACT target/arm-unknown-linux-gnueabihf/release/restedpi AS LOCAL build/restedpi


