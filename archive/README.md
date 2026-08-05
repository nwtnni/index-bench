# Archive of benchmark results

Experiments run on a [Chameleon](https://www.chameleoncloud.org/) compute_icelake_r650 instance ([example](https://www.chameleoncloud.org/hardware/node/sites/tacc/clusters/chameleon/nodes/dde004bf-b99b-4c0a-b2d4-d5537378626a/)).
Each instance has two Intel(R) Xeon(R) Platinum 8380 CPUs, each with:
- 2.30 GHz
- 40 cores
- 120 MiB LLC
- 128 GiB DDR4 3200 DRAM

# osdi-2026

This directory contains peer-reviewed benchmark results referenced
in our [OSDI paper](https://www.usenix.org/conference/osdi26/presentation/ni).
These should be reproducible at the `osdi-2026-artifact-evaluation` git tags for
this repository and [arctic](github.com/nwtnni/arctic).

# all

This directory contains **non-peer-reviewed** benchmark results collected after publication, with
[arctic v0.1.4](https://crates.io/crates/arctic-map/0.1.4)
and a larger set of concurrent Rust baselines.

# sequential

This directory contains **non-peer-reviewed** benchmark results collected after publication,
at one thread, with [arctic v0.1.3](https://crates.io/crates/arctic-map/0.1.3)
and a larger set of sequential and concurrent Rust baselines.
