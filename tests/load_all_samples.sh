#!/bin/bash
for file in ../gltf/glTF-Sample-Models/2.0/**/glTF/*.gltf; do
    echo "$file"
    CARGO_INCREMENTAL=1 cargo run --release -- "$file" -s
done
