#!/bin/bash
set -eux
xvfb-run --auto-servernum --server-args="-screen 0 640x480x24" gltf-viewer "$@"
