use std::fs::File;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;

use anyhow::Context as _;
use anyhow::anyhow;
use perf_event::Builder;
use perf_event::Counter;
use perf_event::Group;
use perf_event::events::Hardware;
use perf_event::events::Software;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    cache_access: u64,
    cache_miss_rate: f64,
    branch: u64,
    branch_miss_rate: f64,
    lock_load: u64,
    l3_hitm: u64,
}

pub(crate) struct Perf {
    group: Group,
    counters: Vec<Counter>,
}

impl Perf {
    pub fn new(cpu: usize) -> Self {
        let mut group = Group::builder().one_cpu(cpu).build_group().unwrap();
        let mut counters = Vec::new();

        let mut template = Builder::new(Software::DUMMY);
        let template = template.one_cpu(cpu);

        for event in [
            Hardware::CACHE_REFERENCES,
            Hardware::CACHE_MISSES,
            Hardware::BRANCH_INSTRUCTIONS,
            Hardware::BRANCH_MISSES,
        ] {
            counters.push(template.event(event).build_with_group(&mut group).unwrap());
        }

        // From Intel SDM Vol. 3B Table 22-37
        for config in [
            0x21D0, // mem_inst_retired.lock_loads
            0x04D2, // mem_load_l3_hit_retired.xsnp_hitm
        ] {
            counters.push(
                template
                    .event(perf_event::events::Raw::new(config))
                    .build_with_group(&mut group)
                    .unwrap(),
            );
        }

        Self { group, counters }
    }

    pub fn start(&mut self) {
        self.group.enable().expect("Perf event enable");
    }

    pub fn stop(&mut self) -> Report {
        self.group.disable().expect("Perf event disable");

        let data = self.group.read().expect("Perf event read");

        let scale =
            data.time_enabled().unwrap().as_secs_f64() / data.time_running().unwrap().as_secs_f64();

        let get = |index: usize| (data[&self.counters[index]] as f64 * scale) as u64;

        let cache_access = get(0);
        let branch = get(2);

        Report {
            cache_access,
            cache_miss_rate: get(1) as f64 / cache_access as f64,
            branch,
            branch_miss_rate: get(3) as f64 / branch as f64,
            lock_load: get(4),
            l3_hitm: get(5),
        }
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
