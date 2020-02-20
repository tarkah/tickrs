FROM debian:stretch

RUN dpkg --add-architecture arm64 && \ 
    apt-get update && \
    apt-get install -y curl build-essential gcc-aarch64-linux-gnu pkg-config

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.40.0

ENV RUSTUP_HOME=/root/.rustup \
    CARGO_HOME=/root/.cargo \
    PATH=/root/.cargo/bin:$PATH \
    RUST_TEST_THREADS=1 \
    PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig

RUN rustup target add aarch64-unknown-linux-gnu
RUN bash -c 'echo -e "[target.aarch64-unknown-linux-gnu]\nlinker = \"aarch64-linux-gnu-gcc\"" > /root/.cargo/config'
