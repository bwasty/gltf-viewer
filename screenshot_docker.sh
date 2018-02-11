#!/bin/bash
set -eu

if [[ ${#} -lt 1 ]]; then
    echo "Usage: ${0} <gltf-file-path> [<additional-arguments>]"
    echo "Example: ${0} ../gltf/Box.glb --count 3 --headless"
    exit 1
fi

gltf_path=$1
gltf_dir=$(dirname "$gltf_path")
gltf_file=$(basename "$gltf_path")
model_name=${gltf_file%.*}
output_file="$gltf_dir/$model_name.png"

docker build -t gltf-viewer .
rm "$output_file" || true
docker run -t --rm -v "$(pwd)/$gltf_dir:/input" \
    gltf-viewer "/input/$gltf_file" -s "/input/$model_name.png" "${@:2}"
# uncomment to run bash interactively
# docker run -it -v "$(pwd)/$gltf_dir:/input" --entrypoint bash gltf-viewer
[ -f "$HOME/.iterm2/imgcat" ] && "$HOME/.iterm2/imgcat" "$output_file"
