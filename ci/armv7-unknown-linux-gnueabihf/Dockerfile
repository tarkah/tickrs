FROM debian:stretch

RUN dpkg --add-architecture armhf && \ 
    apt-get update && \
    apt-get install -y curl build-essential gcc-arm-linux-gnueabihf pkg-config

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.40.0

ENV RUSTUP_HOME=/root/.rustup \
    CARGO_HOME=/root/.cargo \
    PATH=/root/.cargo/bin:$PATH \
    RUST_TEST_THREADS=1 \
    PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig

RUN rustup target add armv7-unknown-linux-gnueabihf
RUN bash -c 'echo -e "[target.armv7-unknown-linux-gnueabihf]\nlinker = \"arm-linux-gnueabihf-gcc\"" > /root/.cargo/config'
