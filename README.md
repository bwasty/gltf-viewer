# gltf-viewer
[![crates.io](https://img.shields.io/crates/v/gltf-viewer.svg)](https://crates.io/crates/gltf-viewer)
[![GitHub release](https://img.shields.io/github/release/bwasty/gltf-viewer.svg)](https://github.com/bwasty/gltf-viewer/releases/latest)
 [![](https://tokei.rs/b1/github/bwasty/gltf-viewer)](https://github.com/Aaronepower/tokei)
 [![Build Status](https://travis-ci.org/bwasty/gltf-viewer.svg?branch=master)](https://travis-ci.org/bwasty/gltf-viewer)
 [![Build status](https://ci.appveyor.com/api/projects/status/51ukh02thpb0r9cf/branch/master?svg=true)](https://ci.appveyor.com/project/bwasty/gltf-viewer/branch/master) <br>
 <!--
 [![Crates.io](https://img.shields.io/crates/d/gltf-viewer.svg)](https://crates.io/crates/gltf-viewer)
 [![Github All Releases](https://img.shields.io/github/downloads/bwasty/gltf-viewer/total.svg)](https://github.com/bwasty/gltf-viewer/releases)
-->

[glTF](https://github.com/KhronosGroup/glTF) 2.0 Viewer written in Rust.

**Current state** : All sample models can be loaded:

<img width="412" alt="gltf-viewer-0.1" src="https://user-images.githubusercontent.com/1647415/30771307-d70dbd26-a044-11e7-9ed1-b0e2ba80198c.png">

Some glTF features are not implemented yet, most notably animations. See [#3](https://github.com/bwasty/gltf-viewer/issues/3) for details.

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
```shell
USAGE:
    gltf-viewer [OPTIONS] <FILE>

OPTIONS:
    -s, --screenshot <FILE>    Create screenshot (PNG)
    -v, --verbose              Enable verbose logging (log level INFO). Can be repeated multiple times to increase log level to DEBUG/TRACE
    -w, --width <WIDTH>        Width in pixels [default: 800]
    -h, --height <HEIGHT>      Height in pixels [default: 600]
        --help                 Prints help information
    -V, --version              Prints version information

ARGS:
    <FILE>    glTF file name
```
Both .gltf and .glb files are supported.
Navigate the scene with `WASD` + Mouse.

### Example
```
$ curl -O https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/master/2.0/Box/glTF-Binary/Box.glb
$ gltf-viewer Box.glb
```
For more models, see [KhronosGroup/glTF-Sample-Models](https://github.com/KhronosGroup/glTF-Sample-Models/tree/master/2.0).

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
