use std::fs::File;
use std::io::BufWriter;
use std::io::Write as _;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context as _;
use cartesian::IterCartesian as _;

fn main() -> anyhow::Result<()> {
    let data = std::fs::read_to_string(std::env::args().nth(1).expect("Expected config file"))?;
    let configs = toml::from_str::<cartesian::Iter<index_bench::Config>>(&data)?;

    let mut out = File::options()
        .create(true)
        .append(true)
        .open("result.ndjson")
        .map(BufWriter::new)?;

    for mut config in configs.iter_cartesian() {
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

        // Preserve ordering for snowflake keys
        if matches!(
            config.workload.key,
                | index_bench::workload::Key::Snowflake
        ) && matches!(config.workload.ycsb.insert_order, ycsb::InsertOrder::Hashed)
        {
            continue;
        }

        // Skip ordering for random keys
        if matches!(
            config.workload.key,
            index_bench::workload::Key::Url
                | index_bench::workload::Key::Email
                | index_bench::workload::Key::Ipv4
                | index_bench::workload::Key::UuidV4
        ) && matches!(
            config.workload.ycsb.insert_order,
            ycsb::InsertOrder::Ordered
        ) {
            continue;
        }

        // Congee only supports u64 keys
        if matches!(config.index.name, index_bench::index::Name::Congee)
            && matches!(
                config.workload.key,
                index_bench::workload::Key::UuidV4
                    | index_bench::workload::Key::Url
                    | index_bench::workload::Key::Email
            )
        {
            continue;
        }

        // Masstree only supports keys up to 256 bytes
        if matches!(config.index.name, index_bench::index::Name::Masstree)
            && matches!(config.workload.key, index_bench::workload::Key::Url)
        {
            continue;
        }

        // Sequential indexes only support one thread
        if matches!(
            config.index.name,
            index_bench::index::Name::ArcticSeq
                | index_bench::index::Name::StdBTreeMap
                | index_bench::index::Name::StdHashMap
        ) && config.global.thread_count > 1
        {
            continue;
        }

        // Clamp record count for URL dataset (contains 38.3M, but leave room for inserts)
        if matches!(config.workload.key, index_bench::workload::Key::Url) {
            config.workload.ycsb.record_count = config.workload.ycsb.record_count.min(30_000_000);
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
        out.flush()?;
    }

    Ok(())
}
