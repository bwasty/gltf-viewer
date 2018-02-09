FROM ubuntu:16.04

RUN apt-get update && apt-get install -y curl gcc xvfb xorg libosmesa6-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY run_xvfb.sh run_xvfb.sh

RUN cargo install --debug

VOLUME /input
WORKDIR /input
ENTRYPOINT /run_xvfb.sh
