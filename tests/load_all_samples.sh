#!/bin/bash
# Loads all sample models and generates screenshots
# NOTE: should be called from crate root!
# Parameters:
mode=${1:-release} # pass `debug` instead to test a debug build
base_dir=${2:-../../glTF-Sample-Models/2.0}
rest_gltf_arguments="${@:3}"

# set -e
result_dir=target/screenshots/$(date '+%Y-%m-%d_%H-%M-%S')
mkdir -p "$result_dir"
export CARGO_INCREMENTAL=1
if [[ "$mode" == "release" ]]; then
    cargo build --release
else
    cargo build
fi

for file in $base_dir/**/glTF/*.gltf; do
    model_name=$(basename "$file" .gltf)
    # shellcheck disable=SC2086
    target/"$mode"/gltf-viewer "$file" -s "$result_dir"/"$model_name".png $rest_gltf_arguments
done

# for file in $base_dir/**/glTF-Binary/*.glb; do
#     model_name=$(basename "$file" .glb)
#     echo "$model_name"
# shellcheck disable=SC2086
#     target/"$mode"/gltf-viewer "$file" -s "$result_dir"/"$model_name"-Binary.png $rest_gltf_arguments
# done

# for file in $base_dir/**/glTF-Embedded/*.gltf; do
#     model_name=$(basename "$file" .gltf)
# shellcheck disable=SC2086
#     target/"$mode"/gltf-viewer "$file" -s "$result_dir"/"$model_name"-Embedded.png $rest_gltf_arguments
# done
