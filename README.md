# gltf-viewer
<!-- [![crates.io](https://img.shields.io/crates/v/gltf-viewer.svg)] (https://crates.io/crates/gltf-viewer) -->
[![Build Status](https://travis-ci.org/bwasty/gltf-viewer.svg?branch=master)](https://travis-ci.org/bwasty/gltf-viewer) <!-- [![](https://tokei.rs/b1/github/bwasty/gltf-viewer)](https://github.com/Aaronepower/tokei)
[![](https://tokei.rs/b1/github/bwasty/gltf-viewer?category=comments)](https://github.com/Aaronepower/tokei) -->

glTF Viewer written in Rust (WIP).
Current state: most simple models can be loaded, but there is no lighting/texturing (normals are used as color):

<img width="311" alt="gltf-viewer" src="https://user-images.githubusercontent.com/1647415/27612520-375828ee-5b97-11e7-97b4-90785cdbfe8e.png">

Install with
```
git clone https://github.com/bwasty/gltf-viewer.git
cd gltf-viewer
cargo install gltf-viewer
```
(the first two steps are temporary - will soon be on crates.io)

Run with
```
gltf-viewer <filename>
```
Both .gltf and .glb files are supported.


### Goals
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
