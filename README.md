# gltf-viewer [![status](https://img.shields.io/badge/glTF-2%2E0-green.svg?style=flat)](https://github.com/KhronosGroup/glTF)
[![crates.io](https://img.shields.io/crates/v/gltf-viewer.svg)](https://crates.io/crates/gltf-viewer)
[![GitHub release](https://img.shields.io/github/release/bwasty/gltf-viewer.svg)](https://github.com/bwasty/gltf-viewer/releases/latest)
 [![](https://tokei.rs/b1/github/bwasty/gltf-viewer)](https://github.com/Aaronepower/tokei)
 [![Build Status](https://travis-ci.org/bwasty/gltf-viewer.svg?branch=master)](https://travis-ci.org/bwasty/gltf-viewer)
 [![Build status](https://ci.appveyor.com/api/projects/status/51ukh02thpb0r9cf/branch/master?svg=true)](https://ci.appveyor.com/project/bwasty/gltf-viewer/branch/master)
 [![Docker build status](https://img.shields.io/docker/build/bwasty/gltf-viewer.svg)](https://hub.docker.com/r/bwasty/gltf-viewer/tags/)
 [![Maintenance](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg)](https://github.com/rust-lang/rfcs/blob/master/text/1824-crates.io-default-ranking.md#maintenance)

Rust [glTF 2.0](https://github.com/KhronosGroup/glTF) viewer, written using the [gltf](https://github.com/gltf-rs/gltf) crate and plain OpenGL.

**Current state**: All [official sample models](https://github.com/KhronosGroup/glTF-Sample-Models/tree/master/2.0) can be loaded and are rendered with the [reference PBR shader](https://github.com/KhronosGroup/glTF-WebGL-PBR). Example: <br>
<img width="412" alt="SciFiHelmet" title="SciFiHelmet" src="https://user-images.githubusercontent.com/1647415/30771307-d70dbd26-a044-11e7-9ed1-b0e2ba80198c.png"><br>
Gallery with all sample models: https://bwasty.github.io/gltf-viewer/0.3.0/

Some glTF features are not yet implemented, most notably **animations**. See [#3](https://github.com/bwasty/gltf-viewer/issues/3) for details.

## Installation
### Binaries (Win/Linux/macOS)
See [Latest Release](https://github.com/bwasty/gltf-viewer/releases/latest)
### From crate (requires [Rust](https://www.rust-lang.org))
```shell
cargo install gltf-viewer
```
Latest version (unstable):
```shell
cargo install --git https://github.com/bwasty/gltf-viewer.git
```
## Usage
```
USAGE:
    gltf-viewer [OPTIONS] <FILE>

OPTIONS:
    -v, --verbose                    Enable verbose logging (log level INFO). Can be repeated up to 3 times to increase
                                     log level to DEBUG/TRACE)
    -s, --screenshot <FILE>          Create screenshot (PNG)
    -w, --width <WIDTH>              Width in pixels [default: 800]
    -h, --height <HEIGHT>            Height in pixels [default: 600]
    -c, --count <COUNT>              Saves N screenshots of size WxH, rotating evenly spaced around the object [default:
                                     1]
        --headless                   Use real headless rendering for screenshots (default is a hidden window)
                                     [EXPERIMENTAL - see README for details]
        --straight                   Position camera in front of model if using default camera (i.e. glTF doesn't
                                     contain a camera or `--cam-index -1` is passed).
        --scene <scene>              Index of the scene to load [default: 0]
        --cam-index <CAM-INDEX>      Use the glTF camera with the given index (starting at 0).
                                     Fallback if there is none: determine 'nice' camera position based on the scene's
                                     bounding box. Can be forced by passing -1.
                                     Note: All other camera options are ignored if this one is given. [default: 0]
        --cam-pos <CAM-POS>          Camera (aka eye) position override as comma-separated Vector3. Example: 1.2,3.4,5.6
        --cam-target <CAM-TARGET>    Camera target (aka center) override as comma-separated Vector3. Example:
                                     1.2,3.4,5.6
        --cam-fovy <CAM-FOVY>        Vertical field of view ('zoom') in degrees. [default: 75]
        --help                       Prints help information
    -V, --version                    Prints version information

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

### wasm/webgl target
Build the wasm web target with this command. You may need to follow setup instructions from
the [wasm-pack website](https://rustwasm.github.io/docs/wasm-pack/).
```
wasm-pack build --target web -- --no-default-features --features use_wasm_bindgen
```
This generates a pkg/ directory with js and wasm files. Use a simple http server (such as `python3 -m http.server`) to load the index.html demo page.
