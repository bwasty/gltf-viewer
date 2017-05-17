# gltf-viewer
glTF Viewer written in Rust (early WIP)

### Goals
* Complete gltF 2.0 support
* Reusable & extensible renderer
* Platforms: Windows/Linux/macOS; Browser via WebAssembly
* Graphics backends: 
  - Vulkan
  - OpenGL ES 3.0 (-> WebGL 2.0 via WebAssembly)
* VR support (Focus: HTC Vive)

### Non-goals
* Support for proprietary APIs like Metal, Direct3D
 - open for contributions though
* Fancy UI (i.e. No more than command line + "switches" for changing scenes, toggling animations etc.)
