# gltf-viewer
[![crates.io](https://img.shields.io/crates/v/gltf-viewer.svg)](https://crates.io/crates/gltf-viewer)
 [![](https://tokei.rs/b1/github/bwasty/gltf-viewer)](https://github.com/Aaronepower/tokei)
 [![Build Status](https://travis-ci.org/bwasty/gltf-viewer.svg?branch=master)](https://travis-ci.org/bwasty/gltf-viewer)

glTF Viewer written in Rust (WIP).
Current state: most simple models can be loaded, but there is no lighting/texturing (normals are used as color):

<img width="311" alt="gltf-viewer" src="https://user-images.githubusercontent.com/1647415/27612520-375828ee-5b97-11e7-97b4-90785cdbfe8e.png">

## Installation
### From crates.io
```shell
cargo install gltf-viewer
```
### From source
```shell
git clone https://github.com/bwasty/gltf-viewer.git
cd gltf-viewer
cargo install gltf-viewer
```

### Additional dependencies (Ubuntu)
`sudo apt-get install cmake libssl-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev` (rough list, to be refined)

## Usage
```shell
gltf-viewer <filename|URL>
```
Both .gltf and .glb files are supported.
Navigate the scene with `WASD` + Mouse.

### Example
```
gltf-viewer https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/master/2.0/BarramundiFish/glTF/BarramundiFish.gltf
```

## Goals
* Complete gltF 2.0 support
* Reusable & extensible renderer
  - may be extracted later for a `gltf-engine` that can be used for 3D apps
* Platforms: Windows, Linux, macOS, Browser (via WebAssembly)
* Graphics backends:
  - OpenGL ES 3.0 (-> WebGL 2.0 via WebAssembly)
  - Vulkan?
* VR support
  * Focus: OpenVR (HTC Vive)
  * WebVR?
