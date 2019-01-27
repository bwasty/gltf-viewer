FROM rust:1.32

WORKDIR /usr/src/gltf-viewer
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo install --path .

RUN apt-get update

RUN apt-get install -y xvfb
COPY run_xvfb.sh run_xvfb.sh
ENTRYPOINT [ "./run_xvfb.sh" ]

# Hint: To try 'real' headless rendering,
# toggle comments on the previous and following block and use the `--headless` parameter
# RUN apt-get install -y libosmesa6-dev
# ENTRYPOINT [ "gltf-viewer" ]
