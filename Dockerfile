FROM rust:1.23

WORKDIR /usr/src/gltf-viewer
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo install

RUN apt-get update

RUN apt-get install -y xvfb
COPY run_xvfb.sh run_xvfb.sh
ENTRYPOINT [ "./run_xvfb.sh" ]

# Hint: To try 'real' headless rendering,
# toggle comments on the previous and following block and use the `--headless` parameter
# RUN apt-get install -y libosmesa6-dev
# ENTRYPOINT [ "gltf-viewer" ]

# Cleanup to keep the image "small"
RUN rm -rf target /usr/local/cargo/registry /var/lib/apt/lists/*
