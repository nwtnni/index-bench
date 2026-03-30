#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

# Once with membarrier.
# mkdir -p raw/reclaim-test/membarrier
# for smr in "default" "smr-seize" "smr-epoch" "smr-disable"; do
#     cargo build --release --features "stat,membarrier,$smr"
#     cargo run --release  --features "stat,membarrier,$smr" -- bench/smr-reclaim.toml
#     mv result.ndjson ./raw/reclaim-test/membarrier/"arctic-reclaim-$smr.ndjson"
# done
# cat ./raw/reclaim-test/membarrier/arctic-reclaim-* > ./raw/reclaim-test/membarrier/reclaim-test.ndjson
#
# And once without...
mkdir -p raw/reclaim-test/no-membarrier
for smr in "default" "smr-disable"; do
    cargo build --release --features "$smr"
    cargo run --release  --features "$smr" -- bench/smr-reclaim.toml
    mv result.ndjson ./raw/reclaim-test/no-membarrier/"arctic-reclaim-$smr.ndjson"
done
cat ./raw/reclaim-test/no-membarrier/arctic-reclaim-* > ./raw/reclaim-test/no-membarrier/reclaim-test.ndjson
