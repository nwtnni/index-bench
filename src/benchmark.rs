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

use crate::config;
use crate::measure;

pub fn run(config: &config::Global) -> anyhow::Result<()> {
    let date = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let topology = &Topology::new().context("Initialize hwloc topology")?;
    let depth = topology
        .depth_for_type(ObjectType::PU)
        .map_err(|error| anyhow!("Failed to get processing unit depth: {:?}", error))?;
    let cores = topology.objects_at_depth(depth).cycle();

    config.numa.bind(topology)?;

    let barrier = &Barrier::new(config.thread_count);

    let mut perf_external = match (env::var("PERF_CTL_FIFO"), env::var("PERF_ACK_FIFO")) {
        (Ok(ctl), Ok(ack)) => Some(measure::perf::Sync::new(ctl, ack)?),
        _ => None,
    };
    let perf_internal = perf_external.is_none();

    if let Some(perf) = &mut perf_external {
        perf.enable()?;
    }

    thread::scope(|scope| -> anyhow::Result<_> {
        let workers = (0..config.thread_count)
            .zip(cores)
            .map(|(thread_id, core)| {
                scope.spawn(move || -> anyhow::Result<_> {
                    crate::THREAD_ID.set(thread_id);

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
                        .then(|| measure::Perf::new(core.os_index().expect("No OS index for core")))
                        .transpose()
                        .context("Initialize perf-event")?;

                    let _ = barrier.wait();
                    let before = measure::Resource::new().context("Get resource usage")?;
                    if let Some(perf) = &mut perf {
                        perf.start().context("Start perf-event")?;
                    }

                    let start = Instant::now();

                    todo!();

                    let total = start.elapsed();

                    let report = perf
                        .as_mut()
                        .map(|perf| perf.stop())
                        .transpose()
                        .context("Stop perf-event")?;
                    let after = measure::Resource::new().context("Get resource usage")?;
                    let _ = barrier.wait();

                    Ok((thread_id, total.as_nanos(), after - before, report))
                })
            })
            .collect::<Vec<_>>();

        let output_workers = workers
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<anyhow::Result<Vec<_>>>()
            .unwrap();

        if let Some(perf) = &mut perf_external {
            perf.disable()?;
        }

        let mut stdout = std::io::stdout().lock();
        serde_json::ser::to_writer(
            &mut stdout,
            &measure::Global {
                date,
                config: config.clone(),
                thread: output_workers
                    .into_iter()
                    .map(|(id, time, resource, perf)| measure::Thread {
                        id,
                        time,
                        resource,
                        perf,
                    })
                    .collect(),
            },
        )?;

        Ok(())
    })
}
