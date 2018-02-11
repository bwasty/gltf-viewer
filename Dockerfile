FROM rust:1.23

WORKDIR /usr/src/gltf-viewer
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo install

RUN apt-get update

RUN apt-get install -y xvfb
# COPY run_xvfb.sh run_xvfb.sh
# ENTRYPOINT [ "./run_xvfb.sh" ]

RUN apt-get install -y libosmesa6-dev mesa-utils
ENTRYPOINT [ "gltf-viewer" ]
