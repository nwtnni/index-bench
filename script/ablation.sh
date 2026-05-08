#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

for level in $(seq 0 4); do
    cargo build --release --no-default-features --features "opt-$level"
    for i in $(seq 10); do
        cargo run --release --no-default-features --features "opt-$level" -- bench/ablation-load.toml
        cargo run --release --no-default-features --features "opt-$level" -- bench/ablation-read-scan.toml
    done
    sed -i"" "s/arctic/arctic-$level/g" result.ndjson
    mv result.ndjson "arctic-$level.ndjson"
done

cat arctic-*.ndjson > ablation.ndjson
