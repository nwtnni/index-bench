#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

for workload in load run; do
    cargo build --release --features "opt-4"
    for i in $(seq 10); do
        cargo run --release --features "opt-4" -- "bench/$workload.toml"
    done
    mv result.ndjson "$workload.ndjson"
done

for workload in latest scan; do
    cargo build --release
    for i in $(seq 10); do
        cargo run --release -- "bench/$workload.toml"
    done
    mv result.ndjson "$workload.ndjson"
done
