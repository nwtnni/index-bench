use std::fs::File;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;

use anyhow::Context as _;
use anyhow::anyhow;
use perf_event::Builder;
use perf_event::Counter;
// use perf_event::events::Cache;
// use perf_event::events::CacheId;
// use perf_event::events::CacheOp;
// use perf_event::events::CacheResult;
use perf_event::events::Hardware;
use perf_event::events::Software;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Report {
    cpu_cycle_count: u64,
    instruction_count: u64,
    // l1d_rm: u64,
    // l1d_ra: u64,
    //
    // l1d_wm: u64,
    // l1d_wa: u64,
    //
    // ll_rm: u64,
    // ll_ra: u64,
    //
    // ll_wm: u64,
    // ll_wa: u64,
}

pub(crate) struct Perf {
    group: perf_event::Group,
    counters: Vec<Counter>,
}

impl Perf {
    pub fn new(cpu: usize) -> anyhow::Result<Self> {
        let mut group = perf_event::Group::builder()
            .one_cpu(cpu)
            .build_group()
            .context("Perf build group")?;
        let mut counters = Vec::new();

        let mut builder = Builder::new(Software::DUMMY);
        builder.one_cpu(cpu);

        counters.push(
            group
                .add(builder.event(Hardware::CPU_CYCLES))
                .context("Perf add cycles")?,
        );
        counters.push(
            group
                .add(builder.event(Hardware::INSTRUCTIONS))
                .context("Perf add instructions")?,
        );

        // for which in [CacheId::L1D, CacheId::LL] {
        //     for operation in [CacheOp::READ, CacheOp::WRITE] {
        //         for result in [CacheResult::MISS, CacheResult::ACCESS] {
        //             let cache = Cache {
        //                 which,
        //                 operation,
        //                 result,
        //             };
        //             counters.push(
        //                 group
        //                     .add(builder.event(cache.clone()))
        //                     .with_context(|| anyhow!("Perf add cache: {:?}", cache))?,
        //             );
        //         }
        //     }
        // }

        Ok(Self { group, counters })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.group.enable().context("Perf event enable")?;
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<Report> {
        self.group.disable().context("Perf event disable")?;

        let data = self.group.read().context("Perf event read")?;

        let scale =
            data.time_enabled().unwrap().as_secs_f64() / data.time_running().unwrap().as_secs_f64();

        let get = |index: usize| (data[&self.counters[index]] as f64 * scale) as u64;

        Ok(Report {
            cpu_cycle_count: get(0),
            instruction_count: get(1),
            // l1d_rm: get(2),
            // l1d_ra: get(3),
            //
            // l1d_wm: get(4),
            // l1d_wa: get(5),
            //
            // ll_rm: get(6),
            // ll_ra: get(7),
            //
            // ll_wm: get(8),
            // ll_wa: get(9),
        })
    }
}

pub(crate) struct Sync {
    ctl: File,
    ack: File,
}

impl Sync {
    pub fn new<C: AsRef<Path>, A: AsRef<Path>>(ctl: C, ack: A) -> anyhow::Result<Self> {
        let ctl = ctl.as_ref();
        let ack = ack.as_ref();

        Ok(Self {
            ctl: File::options()
                .write(true)
                .open(ctl)
                .with_context(|| anyhow!("Failed to open perf control file {}", ctl.display(),))?,
            ack: File::options()
                .read(true)
                .open(ack)
                .with_context(|| anyhow!("Failed to open perf ack file {}", ack.display()))?,
        })
    }

    pub fn enable(&mut self) -> anyhow::Result<()> {
        self.ctl
            .write_all(b"enable\n\0")
            .context("Failed to write to perf ctl file")?;
        self.wait()
    }

    pub fn disable(&mut self) -> anyhow::Result<()> {
        self.ctl
            .write_all(b"disable\n\0")
            .context("Failed to write to perf ctl file")?;
        self.wait()
    }

    fn wait(&mut self) -> anyhow::Result<()> {
        self.ack
            .read_exact(&mut [0u8; 5])
            .context("Failed to read from perf ack file")
    }
}
