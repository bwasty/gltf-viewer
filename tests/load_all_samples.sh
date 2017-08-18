#!/bin/bash
# Loads all sample models and generates screenshots
# NOTE: should be called from crate root!
# Parameters:
mode=${1:-release} # pass `debug` instead to test a debug build

set -e
mkdir -p target/screenshots
export CARGO_INCREMENTAL=1
if [[ "$mode" == "release" ]]; then
    cargo build --release
else
    cargo build
fi

for file in ../gltf/glTF-Sample-Models/2.0/**/glTF/*.gltf; do
    model_name=$(basename "$file" .gltf)
    target/"$mode"/gltf-viewer "$file" -s target/screenshots/"$model_name".png -v
done

# for file in ../gltf/glTF-Sample-Models/2.0/**/glTF-Binary/*.glb; do
#     model_name=$(basename "$file" .glb)
#     echo "$model_name"
#     target/"$mode"/gltf-viewer "$file" -s target/screenshots/"$model_name"-Binary.png
# done

# for file in ../gltf/glTF-Sample-Models/2.0/**/glTF-Embedded/*.gltf; do
#     model_name=$(basename "$file" .gltf)
#     target/"$mode"/gltf-viewer "$file" -s target/screenshots/"$model_name"-Embedded.png
# done
