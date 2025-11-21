use std::fs::File;
use std::io::BufWriter;
use std::io::Write as _;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context as _;
use cartesian::IntoIterCartesian as _;

fn main() -> anyhow::Result<()> {
    let data = std::fs::read_to_string(std::env::args().nth(1).expect("Expected config file"))?;
    let configs = toml::from_str::<cartesian::IntoIter<index_bench::Config>>(&data)?;

    let mut out = File::options()
        .create_new(true)
        .write(true)
        .open("result.ndjson")
        .map(BufWriter::new)?;

    for config in configs.into_iter_cartesian() {
        if (config.workload.ycsb.read_proportion
            + config.workload.ycsb.update_proportion
            + config.workload.ycsb.insert_proportion
            + config.workload.ycsb.scan_proportion
            + config.workload.ycsb.delete_proportion
            + config.workload.ycsb.read_modify_write_proportion
            - 1.0)
            .abs()
            > 1e-5
        {
            continue;
        }

        // HACK: hashing doesn't make sense for k-mer workload, but we also
        // don't want to duplicate the configuration file to avoid one case
        if matches!(config.workload.key, index_bench::workload::Key::Kmer)
            && matches!(config.workload.ycsb.insert_order, ycsb::InsertOrder::Hashed)
        {
            continue;
        }

        // HACK: congee doesn't support string keys
        if matches!(config.index.name, index_bench::index::Name::Congee)
            && matches!(
                config.workload.key,
                index_bench::workload::Key::Url | index_bench::workload::Key::Email
            )
        {
            continue;
        }

        eprintln!("{config:?}");

        let mut child = Command::new(if cfg!(debug_assertions) {
            "target/debug/worker"
        } else {
            "target/release/worker"
        })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Spawn worker")?;

        serde_json::to_writer(child.stdin.as_mut().unwrap(), &config)
            .context("Write config to worker")?;

        out.write_all(&child.wait_with_output()?.stdout)
            .context("Write output to file")?;
        out.write_all(b"\n").context("Write newline to file")?;
    }

    Ok(())
}
