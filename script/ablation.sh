#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

for level in $(seq 0 4); do
    cargo build --release --no-default-features --features "opt-$level"
    cargo run --release --no-default-features --features "opt-$level" -- bench/ablation-load.toml
    cargo run --release --no-default-features --features "opt-$level" -- bench/ablation-run.toml
    sed -i"" "s/arctic/arctic-$level/g" result.ndjson
    mv result.ndjson "arctic-$level.ndjson"
done
