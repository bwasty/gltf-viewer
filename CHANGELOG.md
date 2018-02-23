# Changelog

## [0.3.0] - 2018-02-23
### Added
* re-add keyboard controls - `WASD` and cursor keys
* remaining glTF camera options:
  - orthographic
  - infinite perspective
* camera CLI parameters:
  - `--cam-index` (choose camera from gltf file)
  - `--cam-pos`
  - `--cam-target`
  - `--cam-fovy` (zoom)
* ambient lighting
* vertex colors (`COLOR_0` vertex attribute)
* `--scene` parameter to choose from multiple scenes in a file

### Changed
* default field of view (fovy) from 60˚ to 75˚

### Fixed
* Initial 'nice' camera position (some models were not properly centered)
* Zooming with real mouse wheel

## [0.2.0] - 2018-02-11
### Changed
* replace `WASD+mouse` navigation with `OrbitControls` based on three.js:
  - rotate: left click + drag
  - pan: right click + drag (still a bit buggy after rotation/zoom)
  - zoom: mouse wheel
* background for screenshots is now transparent

### Added
* `--count` parameter (short `-c`) to generate multiple screenshots, rotating evenly spaced around the object (#10)
* `--headless` parameter for real headless screenshot generation. Unfortunately it only works on macOS. For a workaround using `xvfb`/Docker see [here](https://github.com/bwasty/gltf-viewer#headless-screenshot-generation).
   - Docker images are automatically built for each branch & tag: https://hub.docker.com/r/bwasty/gltf-viewer/


## [0.1.0] - 2017-09-23
* **add PBR shading**
* add screenshot generation via parameter (`-s, --screenshot <FILE>`)
  - Note: currently creates a hidden window, as the "headless" mode of `glutin` did not work as expected
* Determine 'nice' initial camera position from bounding box of scene
  - doesn't work in all cases yet
  - glTF cameras are still ignored
* update [gltf](https://github.com/gltf-rs/gltf) crate to 0.9.2
* **remove URL parameter** (HTTP handling needs to be re-implemented differently after large changes to the `gltf` crate)

## [0.0.3] - 2018-07-28
* add binaries for Win/Linux/macOS to releases
* switch from `glfw-rs` to `glutin`
* add performance logging
* internal refactoring & optimizations

## [0.0.2] - 2018-07-15
* Fix shader handling (`cargo install`ed version didn't work as shaders weren't compiled into the binary).

## [0.0.1] - 2018-11-15
* Initial release. Can display most glTF files, but without any lighting or textures.

---
The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).
