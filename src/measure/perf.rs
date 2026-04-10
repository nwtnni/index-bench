use std::fs::File;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;

use anyhow::Context as _;
use anyhow::anyhow;
use perf_event::Builder;
use perf_event::Counter;
use perf_event::Group;
use perf_event::GroupData;
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

pub(crate) struct Perf {
    branch_group: Group,
    branch: Counter,
    branch_miss: Counter,
    // lock: Counter,
    l3_group: Group,
    l3_hit: Counter,
    l3_miss: Counter,
    l3_hit_hitm: Counter,
    l3_miss_hitm: Counter,
}

impl Perf {
    pub fn new(cpu: usize) -> Self {
        let mut template = Builder::new(Software::DUMMY);
        let template = template.one_cpu(cpu);

        let mut branch_group = Group::builder().one_cpu(cpu).build_group().unwrap();
        let branch = branch_group
            .add(template.event(Hardware::BRANCH_INSTRUCTIONS))
            .unwrap();
        let branch_miss = branch_group
            .add(template.event(Hardware::BRANCH_MISSES))
            .unwrap();

        // From Intel SDM Vol. 3B Table 22-37
        // https://perfmon-events.intel.com/platforms/icelakex/core-events/core/#core-events
        // MEM_INST_RETIRED.LOCK_LOADS
        // let lock = template
        //     .event(perf_event::events::Raw::new(0x21D0))
        //     .build()
        //     .unwrap();

        let mut l3_group = Group::builder()
            .one_cpu(cpu)
            .pinned(true)
            .build_group()
            .unwrap();

        // let lock = template
        //     .event(perf_event::events::Raw::new(0x21D0))
        //     .build()
        //     .unwrap();

        // MEM_LOAD_RETIRED.L3_HIT
        let l3_hit = l3_group
            .add(template.event(perf_event::events::Raw::new(0x04D1)))
            .unwrap();
        // MEM_LOAD_RETIRED.L3_MISS
        let l3_miss = l3_group
            .add(template.event(perf_event::events::Raw::new(0x20D1)))
            .unwrap();
        // MEM_LOAD_L3_HIT_RETIRED.XSNP_FWD
        let l3_hit_hitm = l3_group
            .add(template.event(perf_event::events::Raw::new(0x04D2)))
            .unwrap();
        // MEM_LOAD_L3_MISS_RETIRED.REMOTE_HITM
        let l3_miss_hitm = l3_group
            .add(template.event(perf_event::events::Raw::new(0x04D3)))
            .unwrap();

        Self {
            branch_group,
            branch,
            branch_miss,
            // lock,
            l3_group,
            l3_hit,
            l3_miss,
            l3_hit_hitm,
            l3_miss_hitm,
        }
    }

    pub fn start(&mut self) {
        self.branch_group.enable().unwrap();
        // self.lock.enable().unwrap();
        self.l3_group.enable().unwrap();
    }

    pub fn stop(&mut self) -> Report {
        self.branch_group.disable().unwrap();
        // self.lock.disable().unwrap();
        self.l3_group.disable().unwrap();

        let scale = |data: &GroupData| -> f64 {
            data.time_running().unwrap().as_nanos() as f64
                / data.time_enabled().unwrap().as_nanos() as f64
        };

        let get = |data: &GroupData, counter: &Counter, scale: f64| -> u64 {
            (data[counter] as f64 / scale) as u64
        };

        let branch_data = self.branch_group.read().unwrap();
        let branch_scale = scale(&branch_data);
        let branch = get(&branch_data, &self.branch, branch_scale);
        let branch_miss_rate =
            get(&branch_data, &self.branch_miss, branch_scale) as f64 / branch as f64;

        // let lock_load_data = self.lock.read_count_and_time().unwrap();
        // let lock_load_scale =
        //     lock_load_data.time_running as f64 / lock_load_data.time_enabled as f64;
        // let lock_load = (lock_load_data.count as f64 / lock_load_scale) as u64;

        let l3_data = self.l3_group.read().unwrap();
        let l3_scale = scale(&l3_data);
        let l3_miss = get(&l3_data, &self.l3_miss, l3_scale);
        let l3 = get(&l3_data, &self.l3_hit, l3_scale) + l3_miss;
        let l3_miss_rate = l3_miss as f64 / l3 as f64;
        let l3_hitm_rate = (get(&l3_data, &self.l3_hit_hitm, l3_scale)
            + get(&l3_data, &self.l3_miss_hitm, l3_scale)) as f64
            / l3 as f64;

        let report = Report {
            branch,
            branch_miss_rate,
            // lock_load,
            l3,
            l3_miss_rate,
            l3_hitm_rate,
        };

        self.branch_group.reset().unwrap();
        // self.lock.reset().unwrap();
        self.l3_group.reset().unwrap();
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
