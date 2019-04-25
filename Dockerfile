FROM rust:1.34 as build

WORKDIR /usr/src/gltf-viewer
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release

FROM debian:stretch-20190326-slim
RUN apt-get update
RUN apt-get install -y \
    xvfb \
    libxcursor1 \
    libxrandr2 \
    libxi6

COPY --from=build /usr/src/gltf-viewer/target/release/gltf-viewer /bin/gltf-viewer
COPY run_xvfb.sh run_xvfb.sh
ENTRYPOINT [ "./run_xvfb.sh" ]

# Hint: To try 'real' headless rendering,
# toggle comments on the previous and following block and use the `--headless` parameter
# RUN apt-get install -y libosmesa6-dev
# ENTRYPOINT [ "gltf-viewer" ]
