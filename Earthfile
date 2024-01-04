VERSION 0.7
FROM --platform=$BUILDPLATFORM debian:12
WORKDIR /root
ENV DEBIAN_FRONTEND="noninteractive"
ENV PATH=/root/.cargo/bin:$PATH
ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

builder:
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
          xz-utils \
          gcc-arm-linux-gnueabihf

  RUN apt-get install -y \
                      libc6 \
                      libncurses5 \
                      libstdc++6 

  RUN apt-get install -y \
                      libc6-armel-cross \
                      libc6-dev-armel-cross \
                      binutils-arm-linux-gnueabi \
                      libncurses5-dev \
                      build-essential \
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

  RUN apt-get update
  RUN apt-get install -y apt-utils && dpkg-reconfigure apt-utils
  RUN apt-get install -y gcc-arm-linux-gnueabihf

  RUN curl https://sh.rustup.rs -o rustup.sh && \
    sh rustup.sh --default-toolchain stable -y

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


