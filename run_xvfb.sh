#!/bin/bash
model_name=$(basename "$1" .glb)
xvfb-run --auto-servernum --server-args="-screen 0 640x480x24" gltf-viewer $1 -s /input/$model_name.png
