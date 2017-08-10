#!/bin/bash
set -e
mkdir -p target/screenshots
for file in ../gltf/glTF-Sample-Models/2.0/**/glTF/*.gltf; do
    model_name=$(basename "$file" .gltf)
    export RUST_BACKTRACE=1
    CARGO_INCREMENTAL=1 cargo run -- "$file" -s target/screenshots/"$model_name".png
done
