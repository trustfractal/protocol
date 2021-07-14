FROM rust:latest
MAINTAINER Boy Maas <boy.maas@gmail.com>

# Install musl-gcc
RUN apt-get update && \
    apt-get install -y clang && \
    rm -rf /var/lib/apt/lists/*

# We need this to build a static binary

RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rustfmt

# See: https://www.lpalmieri.com/posts/fast-rust-docker-builds/
RUN cargo install cargo-chef

WORKDIR /source

CMD ["bash"]
