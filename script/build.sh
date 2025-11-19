#!/usr/bin/env bash

# https://stackoverflow.com/a/246128
ROOT=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

git submodule update --init --recursive

cd "$ROOT/../extern/mimalloc_rust/libmimalloc-sys/c_src/mimalloc/v3/"
mkdir -p build && cd build
cmake ..
make -j mimalloc-static

cd "$ROOT/../extern/IndexResearch/util/"
git apply ../util.patch || true
