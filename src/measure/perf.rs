use std::fs::File;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;

use anyhow::Context as _;
use anyhow::anyhow;
use perf_event::Builder;
use perf_event::Counter;
use perf_event::ReadFormat;
use perf_event::events::Hardware;
use perf_event::events::Software;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    branch: u64,
    branch_miss_rate: f64,
    // lock_load: u64,
    l3: u64,
    l3_hitm_rate: f64,
    l3_miss_rate: f64,
}

pub(crate) struct Perf(Vec<Counter>);

impl Perf {
    pub fn new() -> Self {
        let mut template = Builder::new(Software::DUMMY);
        let template = template
            .inherit(true)
            .pinned(true)
            .read_format(ReadFormat::empty());

        let branch = template
            .event(Hardware::BRANCH_INSTRUCTIONS)
            .build()
            .unwrap();
        let branch_miss = template.event(Hardware::BRANCH_MISSES).build().unwrap();

        // From Intel SDM Vol. 3B Table 22-37
        // https://perfmon-events.intel.com/platforms/icelakex/core-events/core/#core-events
        // MEM_INST_RETIRED.LOCK_LOADS
        // let lock = template
        //     .event(perf_event::events::Raw::new(0x21D0))
        //     .build()
        //     .unwrap();
        //
        // MEM_LOAD_RETIRED.L3_HIT
        let l3_hit = template
            .event(perf_event::events::Raw::new(0x04D1))
            .build()
            .unwrap();
        // MEM_LOAD_RETIRED.L3_MISS
        let l3_miss = template
            .event(perf_event::events::Raw::new(0x20D1))
            .build()
            .unwrap();
        // MEM_LOAD_L3_HIT_RETIRED.XSNP_FWD
        let l3_hit_hitm = template
            .event(perf_event::events::Raw::new(0x04D2))
            .build()
            .unwrap();
        // MEM_LOAD_L3_MISS_RETIRED.REMOTE_HITM
        let l3_miss_hitm = template
            .event(perf_event::events::Raw::new(0x04D3))
            .build()
            .unwrap();

        Self(vec![
            branch,
            branch_miss,
            // lock,
            l3_hit,
            l3_miss,
            l3_hit_hitm,
            l3_miss_hitm,
        ])
    }

    pub fn enable(&mut self) {
        self.0.iter_mut().for_each(|counter| {
            counter.enable().unwrap();
            // Make sure counter is scheduled
            counter.read().unwrap();
        });
    }

    pub fn disable(&mut self) -> Report {
        let mut get = |index: usize| {
            let count = self.0[index].read().unwrap();
            self.0[index].disable().unwrap();
            count

            // let data = self.0[index].read_count_and_time().unwrap();
            // dbg!(data.time_running as f64 / data.time_enabled as f64);
            // (data.count as f64 * data.time_enabled as f64 / data.time_running as f64) as u64
        };

        let branch = get(0);
        let branch_miss_rate = get(1) as f64 / branch as f64;

        // let lock_load = get(2);
        let l3_miss = get(3);
        let l3 = get(2) + l3_miss;
        let l3_miss_rate = l3_miss as f64 / l3 as f64;
        let l3_hitm_rate = (get(4) as f64 + get(5) as f64) / l3 as f64;

        let report = Report {
            branch,
            branch_miss_rate,
            // lock_load,
            l3,
            l3_miss_rate,
            l3_hitm_rate,
        };

        self.0
            .iter_mut()
            .for_each(|counter| counter.reset().unwrap());
        report
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
