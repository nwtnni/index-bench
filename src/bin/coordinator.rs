use std::fs::File;
use std::io::BufWriter;
use std::io::Write as _;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context as _;
use cartesian::IntoIterCartesian as _;
use index_bench::ConfigCartesian;

fn main() -> anyhow::Result<()> {
    let data = std::fs::read_to_string(std::env::args().nth(1).expect("Expected config file"))?;
    let configs = toml::from_str::<ConfigCartesian>(&data)?;

    let mut out = File::options()
        .create_new(true)
        .write(true)
        .open("result.ndjson")
        .map(BufWriter::new)?;

    for config in configs.into_iter_cartesian() {
        eprintln!("{config:?}");

        let mut child = Command::new("./target/release/worker")
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
