# gltf-viewer
[![crates.io](https://img.shields.io/crates/v/gltf-viewer.svg)](https://crates.io/crates/gltf-viewer) [![Build Status](https://travis-ci.org/bwasty/gltf-viewer.svg?branch=master)](https://travis-ci.org/bwasty/gltf-viewer) [![](https://tokei.rs/b1/github/bwasty/gltf-viewer)](https://github.com/Aaronepower/tokei)

glTF Viewer written in Rust (WIP).
Current state: most simple models can be loaded, but there is no shading/texturing (normals are used as color).

Install with
```
cargo install gltf-viewer
```

Run with
```
gltf-viewer <filename>
```
Both .gltf and .glb files are supported.

### Goals
* Complete gltF 2.0 support
* Reusable & extensible renderer
* Platforms: Windows/Linux/macOS; Browser via WebAssembly
* Graphics backends:
  - OpenGL ES 3.0 (-> WebGL 2.0 via WebAssembly)
  - OpenGL 4.1+
  - Vulkan?
* VR support (Focus: HTC Vive)

### Non-goals
* Fancy UI (i.e. No more than command line + "switches" for changing scenes, toggling animations etc.)
