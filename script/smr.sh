#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

mkdir -p ndjson/smr
for feature in "default" "smr-disable" "smr-seize" "smr-epoch"; do
    cargo build --release --features "$feature"
    cargo run --release  --features "$feature" -- bench/smr-box.toml
    mv result.ndjson ./ndjson/smr/"arctic-$feature.ndjson"
done

cat arctic-*.ndjson > smr.ndjson
gzip smr.ndjson
