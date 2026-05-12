#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

for feature in "default" "smr-disable" "smr-seize" "smr-epoch"; do
    cargo build --release --features "$feature,stat-garbage"
    cargo run --release  --features "$feature,stat-garbage" -- bench/smr.toml
    mv result.ndjson "arctic-$feature.ndjson"
done

cat arctic-*.ndjson > smr.ndjson
