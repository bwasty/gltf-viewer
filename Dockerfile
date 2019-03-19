FROM rust:1.32

RUN apt-get update

RUN apt-get install -y xvfb
# COPY run_xvfb.sh run_xvfb.sh
# ENTRYPOINT [ "./run_xvfb.sh" ]

RUN apt-get install -y libosmesa6-dev mesa-utils

WORKDIR /usr/src/gltf-viewer
RUN mkdir src && echo "// dummy file" > src/main.rs
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo install --path . || true

COPY src src
RUN cargo install --path .

ENTRYPOINT [ "gltf-viewer" ]
