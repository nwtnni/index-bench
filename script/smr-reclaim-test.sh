#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

mkdir -p ndjson/reclaim-test
for feature in "default" "smr-seize" "smr-epoch"; do
    cargo build --release --features "$feature"
    cargo run --release  --features "$feature" -- bench/smr-reclaim.toml
    mv result.ndjson ./ndjson/reclaim-test/"arctic-reclaim-$feature.ndjson"
done
cat ./ndjson/reclaim-test/arctic-reclaim-* > ./ndjson/reclaim-test/reclaim-test.ndjson
