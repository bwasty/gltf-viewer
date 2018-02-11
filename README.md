# gltf-viewer [![status](https://img.shields.io/badge/glTF-2%2E0-green.svg?style=flat)](https://github.com/KhronosGroup/glTF)
[![crates.io](https://img.shields.io/crates/v/gltf-viewer.svg)](https://crates.io/crates/gltf-viewer)
[![GitHub release](https://img.shields.io/github/release/bwasty/gltf-viewer.svg)](https://github.com/bwasty/gltf-viewer/releases/latest)
 [![](https://tokei.rs/b1/github/bwasty/gltf-viewer)](https://github.com/Aaronepower/tokei)
 [![Build Status](https://travis-ci.org/bwasty/gltf-viewer.svg?branch=master)](https://travis-ci.org/bwasty/gltf-viewer)
 [![Build status](https://ci.appveyor.com/api/projects/status/51ukh02thpb0r9cf/branch/master?svg=true)](https://ci.appveyor.com/project/bwasty/gltf-viewer/branch/master)
 [![Docker build status](https://img.shields.io/docker/build/bwasty/gltf-viewer.svg)](https://hub.docker.com/r/bwasty/gltf-viewer/tags/)

Rust [glTF 2.0](https://github.com/KhronosGroup/glTF) viewer, written using the [gltf](https://github.com/gltf-rs/gltf) crate and plain OpenGL.

**Current state**: All [official sample models](https://github.com/KhronosGroup/glTF-Sample-Models/tree/master/2.0) can be loaded and are rendered with the [reference PBR shader](https://github.com/KhronosGroup/glTF-WebGL-PBR). Example: <br>
<img width="412" alt="SciFiHelmet" title="SciFiHelmet" src="https://user-images.githubusercontent.com/1647415/30771307-d70dbd26-a044-11e7-9ed1-b0e2ba80198c.png"><br>
Some glTF features are not yet implemented, most notably **animations**. See [#3](https://github.com/bwasty/gltf-viewer/issues/3) for details.

## Installation
### Binaries (Win/Linux/macOS)
See [Latest Release](https://github.com/bwasty/gltf-viewer/releases/latest)
### From crate (requires [Rust](https://www.rust-lang.org))
```shell
cargo install gltf-viewer
```
or
```shell
git clone https://github.com/bwasty/gltf-viewer.git
cd gltf-viewer
cargo install gltf-viewer
```
<!--
#### Additional dependencies (Ubuntu)
`sudo apt-get install libssl-dev`
-->
## Usage
```
USAGE:
    gltf-viewer [OPTIONS] <FILE>

OPTIONS:
    -s, --screenshot <FILE>    Create screenshot (PNG)
    -v, --verbose              Enable verbose logging (log level INFO). Can be repeated multiple times to increase log
                               level to DEBUG/TRACE)
    -w, --width <WIDTH>        Width in pixels [default: 800]
    -h, --height <HEIGHT>      Height in pixels [default: 600]
    -c, --count <COUNT>        Saves N screenshots of size WxH, rotating evenly spaced around the object [default: 1]
        --headless             Use real headless rendering for screenshots (Default is a hidden window) [EXPERIMENTAL]
        --help                 Prints help information
    -V, --version              Prints version information

ARGS:
    <FILE>    glTF file name
```
Both .gltf and .glb files are supported.
Navigate the scene with the mouse: Rotate with left click + drag, pan with right click + drag, zoom with mouse wheel.

### Example
```
$ curl -O https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/master/2.0/Box/glTF-Binary/Box.glb
$ gltf-viewer Box.glb
```

### Headless screenshot generation
Proper headless screenshot generation with the `--headless` flag currently only works on macOS.
To work around that, a Docker setup that uses `xvfb` is provided. Usage examples:
```
# Build docker image and run it with the gltf mounted in a volume.
# The image will be saved next to the gltf file.
./screenshot_docker.sh Box.glb
./screenshot_docker.sh ../models/Box.gltf -w 1920 -h 1080 --count 3 -vv
# Use pre-built docker image from Docker Hub
DOCKER_IMAGE=bwasty/gltf-viewer ./screenshot_docker.sh Box.glb
```

Alternatively, you can also install `xvfb` and use `./run_xvfb.sh` directly (Linux only).

## Goals
* Complete gltF 2.0 support
* Reusable & extensible renderer
  - may be extracted later for a `gltf-engine` that can be used for 3D apps
* Platforms: Windows, Linux, macOS, Browser (via WebAssembly)
* Graphics backends:
  - OpenGL ES 3.0 (-> WebGL 2.0 via WebAssembly)
    - currently OpenGL 3.3 is used
  - Vulkan?
* VR support
  * Focus: OpenVR (HTC Vive)
  * WebVR?
