#!/bin/bash
curl -O https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/master/2.0/Box/glTF-Binary/Box.glb
DOCKER_IMAGE=bwasty/gltf-viewer:headless_glutin_issue ./screenshot_docker.sh Box.glb --headless -vv
