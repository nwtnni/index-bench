#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

for workload in load run latest scan; do
    cargo build --release
    cargo run --release -- "bench/$workload.toml"
    mv result.ndjson "$workload.ndjson"
done
