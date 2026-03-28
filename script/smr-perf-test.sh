#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset
set -o xtrace

# Once with membarrier.
mkdir -p raw/reclaim-test/membarrier/perf/folded
for smr in "default" "smr-seize" "smr-epoch" "smr-disable"; do
    cargo build --release --features "membarrier,$smr"
    ./perf-record.sh cargo run --release  --features "membarrier,$smr" -- bench/smr-box-single.toml
    mv out.folded raw/reclaim-test/membarrier/perf/folded/"arctic-reclaim-$smr.folded"
    mv out.svg raw/reclaim-test/membarrier/perf/"arctic-reclaim-$smr.svg"
done

# And once without...
mkdir -p raw/reclaim-test/no-membarrier/perf/folded
for smr in "default" "smr-seize" "smr-epoch" "smr-disable"; do
    cargo build --release --features "$smr"
    ./perf-record.sh cargo run --release  --features "$smr" -- bench/smr-box-single.toml
    mv out.folded raw/reclaim-test/no-membarrier/perf/folded/"arctic-reclaim-$smr.folded"
    mv out.svg raw/reclaim-test/no-membarrier/perf/"arctic-reclaim-$smr.svg"
done

# Now get the differential flamegraphs.
#
# https://docs.rs/inferno/0.12.6/inferno/#differential-flame-graphs
mkdir -p raw/reclaim-test/perf-cmp/folded
for smr in "default" "smr-seize"; do
    # Use the folds we saved to gen differential fold.
    ~/.cargo/bin/inferno-diff-folded raw/reclaim-test/membarrier/perf/folded/"arctic-reclaim-$smr.folded" raw/reclaim-test/no-membarrier/perf/folded/"arctic-reclaim-$smr.folded" > raw/reclaim-test/perf-cmp/folded/"arctic-reclaim-$smr.folded"

    # Get both diffs in svg.
    cat raw/reclaim-test/perf-cmp/folded/"arctic-reclaim-$smr.folded" | ~/.cargo/bin/inferno-flamegraph raw/reclaim-test/perf-cmp/"arctic-reclaim-$smr.folded" >  raw/reclaim-test/perf-cmp/"arctic-reclaim-$smr-diff1.svg"
    cat raw/reclaim-test/perf-cmp/folded/"arctic-reclaim-$smr.folded" | ~/.cargo/bin/inferno-flamegraph raw/reclaim-test/perf-cmp/"arctic-reclaim-$smr.folded" --negate >  raw/reclaim-test/perf-cmp/"arctic-reclaim-$smr-diff2.svg"
done
