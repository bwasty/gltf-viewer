#!/bin/bash
for file in ../../gltf/glTF-Sample-Models/2.0/**/glTF/*.gltf; do
    echo "$file"
    gltf-viewer "$file" -s
done
