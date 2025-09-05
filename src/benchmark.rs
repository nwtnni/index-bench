use std::env;
use std::sync::Barrier;
use std::thread;
use std::time::Instant;
use std::time::SystemTime;

use anyhow::Context as _;
use anyhow::anyhow;
use hwlocality::Topology;
use hwlocality::cpu::binding::CpuBindingFlags;
use hwlocality::object::types::ObjectType;
use serde::Deserialize;
use serde::Serialize;

use crate::Index;
use crate::index::Handle as _;
use crate::measure;

pub fn run<I: Index>(config: crate::Config) -> anyhow::Result<measure::Global> {
    let date = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    // FIXME: support other benchmarks
    let benchmark = Config::YcsbLoad(config.ycsb.clone());

    let topology = &Topology::new().context("Initialize hwloc topology")?;
    let depth = topology
        .depth_for_type(ObjectType::PU)
        .map_err(|error| anyhow!("Failed to get processing unit depth: {:?}", error))?;
    let cores = topology.objects_at_depth(depth).cycle();

    config.global.numa.bind(topology)?;

    let barrier = &Barrier::new(config.global.thread_count);

    let mut perf_external = match (env::var("PERF_CTL_FIFO"), env::var("PERF_ACK_FIFO")) {
        (Ok(ctl), Ok(ack)) => Some(measure::perf::Sync::new(ctl, ack)?),
        _ => None,
    };
    let perf_internal = perf_external.is_none();

    let mut map = I::new();

    let operation_count = benchmark.operation_count(config.global.thread_count) as u64;

    if let Some(perf) = &mut perf_external {
        perf.enable()?;
    }

    let threads = thread::scope(|scope| -> anyhow::Result<_> {
        let benchmark = &benchmark;
        let map = &map;

        (0..config.global.thread_count)
            .zip(cores)
            .map(|(thread_id, core)| {
                scope.spawn(move || -> anyhow::Result<_> {
                    crate::THREAD_ID.set(thread_id);

                    let core_id = core.os_index().expect("No OS index for core");

                    log::debug!("Pin thread {thread_id} to core {core}");

                    topology
                        .bind_cpu(
                            core.cpuset().expect("No cpuset for core"),
                            CpuBindingFlags::THREAD
                                | CpuBindingFlags::STRICT
                                | CpuBindingFlags::NO_MEMORY_BINDING,
                        )
                        .context("Bind thread to CPU")?;

                    let mut perf = perf_internal
                        .then(|| measure::Perf::new(core_id))
                        .transpose()
                        .context("Initialize perf-event")?;

                    let mut loader = match benchmark {
                        Config::YcsbLoad(workload) => {
                            workload.loader(config.global.thread_count, thread_id)
                        }
                    };

                    let mut map = map.pin();

                    let _ = barrier.wait();
                    let before = measure::Resource::new().context("Get resource usage")?;
                    if let Some(perf) = &mut perf {
                        perf.start().context("Start perf-event")?;
                    }

                    let start = Instant::now();

                    while let Some(key) = loader.next_key() {
                        let id = key.id();
                        map.insert(id, id as u32);
                    }

                    let time = start.elapsed();

                    let perf_report = perf
                        .as_mut()
                        .map(|perf| perf.stop())
                        .transpose()
                        .context("Stop perf-event")?;
                    let after = measure::Resource::new().context("Get resource usage")?;
                    let index_report = map.report();

                    let _ = barrier.wait();

                    Ok(measure::Thread {
                        id: thread_id,
                        core: core_id,
                        time: time.as_nanos(),
                        operation_count,
                        resource: after - before,
                        perf: perf_report,
                        index: index_report,
                    })
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<anyhow::Result<Vec<_>>>()
    })?;

    if let Some(perf) = &mut perf_external {
        perf.disable()?;
    }

    Ok(measure::Global {
        date,
        config,
        output: measure::Process {
            index: map.report(),
            thread: threads,
        },
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "benchmark", rename_all = "snake_case")]
pub enum Config {
    YcsbLoad(ycsb::Workload),
}

impl Config {
    fn operation_count(&self, thread_count: usize) -> usize {
        match self {
            Config::YcsbLoad(workload) => workload.record_count() / thread_count,
        }
    }
}
