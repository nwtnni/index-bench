use core::mem;
use std::env;
use std::io;
use std::sync::Barrier;
use std::thread;
use std::time::Instant;
use std::time::SystemTime;

use anyhow::Context as _;
use anyhow::anyhow;
use hwloc2::Topology;

use crate::config;
use crate::measure;

pub fn run(config: &config::Global) -> anyhow::Result<()> {
    let date = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let topology = Topology::new().ok_or_else(|| anyhow!("Failed to retrieve hwloc2 topology"))?;
    let depth = topology
        .depth_for_type(&hwloc2::ObjectType::PU)
        .map_err(|error| anyhow!("Failed to get processing unit depth: {:?}", error))?;
    let cores = topology
        .objects_at_depth(depth)
        .into_iter()
        .map(|core| core.os_index())
        .collect::<Vec<_>>();

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
            .map(|thread_id| {
                let cores = &cores;

                scope.spawn(move || -> anyhow::Result<_> {
                    crate::THREAD_ID.set(thread_id);

                    let core = cores[thread_id % cores.len()];

                    let set = unsafe {
                        // Seems like we should use `MaybeUninit<libc::cpu_set_t>`,
                        // but `CPU_ZERO` macro takes `&mut`, not `*mut`.
                        let mut set = mem::zeroed::<libc::cpu_set_t>();
                        libc::CPU_ZERO(&mut set);
                        libc::CPU_SET(core as usize, &mut set);
                        set
                    };

                    log::debug!("Pin thread {} to core {}", thread_id, core);

                    // `hwloc2::Topology::set_cpubind_for_thread` takes `&mut self`,
                    // so just call `sched_setaffinity` ourselves.
                    if unsafe { libc::sched_setaffinity(0, libc::CPU_SETSIZE as usize, &set) } != 0
                    {
                        return Err(io::Error::last_os_error())
                            .with_context(|| anyhow!("sched_setaffinity({})", core));
                    }

                    let mut perf = perf_internal
                        .then(|| measure::Perf::new(core as usize))
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
