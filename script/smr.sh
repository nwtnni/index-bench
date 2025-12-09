#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

for feature in "default" "smr-disable" "smr-epoch"; do
    cargo build --release --features "$feature"
    cargo run --release  --features "$feature" -- bench/smr.toml
    mv result.ndjson "arctic-$feature.ndjson"
done

cat arctic-*.ndjson > smr.ndjson
gzip smr.ndjson
