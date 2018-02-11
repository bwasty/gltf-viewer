#!/bin/bash
set -eu

if [[ ${#} -lt 1 ]]; then
    echo "Usage: ${0} <gltf-file-path> [<additional-arguments>]"
    echo "Example: ${0} ../gltf/Box.glb --count 3 --headless"
    echo "You can pull a docker image instead of building locally by setting DOCKER_IMAGE."
    echo "Example: DOCKER_IMAGE=bwasty/gltf-viewer:latest ${0} ../gltf/Box.glb"
    exit 1
fi

gltf_path=$1
gltf_dir=$(dirname "$gltf_path")
gltf_file=$(basename "$gltf_path")
model_name=${gltf_file%.*}
output_file="$gltf_dir/$model_name.png"

if [[ ! -z "${DOCKER_IMAGE:-}" ]]; then
    docker pull "$DOCKER_IMAGE"
    image="$DOCKER_IMAGE"
else
    docker build -t gltf-viewer .
    image=gltf-viewer
fi

rm "$output_file" || true
docker run -t --rm -v "$(pwd)/$gltf_dir:/input" \
    $image "/input/$gltf_file" -s "/input/$model_name.png" "${@:2}"
echo "Running bash on container for debugging"
docker run -it -v "$(pwd)/$gltf_dir:/input" --entrypoint bash $image
# -> xvfb-run --auto-servernum --server-args="-screen 0 640x480x24" glxinfo | grep OpenGL
# [ -f "$HOME/.iterm2/imgcat" ] && "$HOME/.iterm2/imgcat" "$output_file"
