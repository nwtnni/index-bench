use std::fs::File;
use std::io::BufWriter;
use std::io::Write as _;

use cartesian::Cartesian;
use index_bench::config;
use serde::Deserialize;
use serde::Serialize;

#[derive(Cartesian)]
#[rustfmt::skip]
#[derive(Serialize, Deserialize)]
struct Config {
    thread_count: usize,

    numa: config::Numa,

    ycsb: ycsb::Workload,
}

fn main() -> anyhow::Result<()> {
    let data = std::fs::read_to_string(std::env::args().nth(1).expect("Expected config file"))?;
    let configs = toml::from_str::<ConfigCartesian>(&data)?;

    let mut out = File::options()
        .create_new(true)
        .write(true)
        .open("result.ndjson")
        .map(BufWriter::new)?;

    for config in configs.cartesian() {
        let measurement = index_bench::run(
            index_bench::config::Global::new(config.thread_count, config.numa),
            index_bench::Benchmark::YcsbLoad(config.ycsb),
        )?;

        serde_json::to_writer(&mut out, &measurement)?;
        out.write_all(b"\n")?;
    }

    Ok(())
}
