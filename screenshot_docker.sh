#!/bin/bash
gltf_path=$1
docker build -t gltf-viewer .
mkdir -p docker_input
cp "$gltf_path" docker_input
# docker run -v "$(pwd)/docker_input:/input" gltf-viewer xvfb-run --auto-servernum --server-args="-screen 0 640x480x24" gltf-viewer -vv "/input/$(basename "$gltf_path")"
# docker run -it -v "$(pwd)/docker_input:/input" gltf-viewer bash
docker run -v "$(pwd)/docker_input:/input" gltf-viewer -vv "/input/$(basename "$gltf_path")"
