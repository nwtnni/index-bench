# index-bench

Benchmark harness for concurrent shared memory index data structures.

## Getting Started Instructions

The main setup script is `script/setup.sh`, which we typically use to
set up a fresh [Chameleon Cloud](https://www.chameleoncloud.org/)
`compute_icelake_r650` instance running a `CC-Ubuntu22.04` image.

This repository uses a [Nix](https://nixos.org/) flake and [direnv](https://direnv.net/)
to set up a reproducible build environment. The setup script installs
these, downloads some benchmark datasets, and clones some submodules
and two macrobenchmark repositories.

Once everything is installed, build the benchmark runner with

```
cargo build --release
```

The runner takes a path to a configuration file (stored in `bench/`),
and appends structured JSON output to a `result.ndjson` file. For example,
to run YCSB load workloads:

```
cargo run --release -- ./bench/load.toml
```

Scripts for plotting and visualizing `result.ndjson` are stored in `plot/`.
The main script for interactively inspecting the data is `plot/dashboard.py`,
which opens a web interface.

## Detailed Instructions

TODO
