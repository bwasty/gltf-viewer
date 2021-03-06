#!/bin/bash
# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src
    src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # NOTE: times out on travis - lto at fault?
    cross rustc --target "$TARGET" --release -- -C lto

    cp target/"$TARGET"/release/gltf-viewer "$stage"/

    cd "$stage"
    tar czf "$src"/"$CRATE_NAME"-"$TRAVIS_TAG"-"$TARGET".tar.gz -- *
    cd "$src"

    rm -rf "$stage"
}

main
