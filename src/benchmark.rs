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
use rand::SeedableRng as _;
use rand::rngs::SmallRng;

use crate::Index;
use crate::index;
use crate::index::IndexPin as _;
use crate::index::IndexSend as _;
use crate::index::Key as _;
use crate::measure;
use crate::workload::KeyDistribution;

pub fn run<K: KeyDistribution, V: index::Value, I: Index<K::Key, V, H>, H: index::Hasher>(
    config: crate::Config,
) -> anyhow::Result<measure::Global> {
    let date = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let operation_count_per_thread = config
        .workload
        .operation_count_per_thread(config.global.thread_count);

    let topology = &Topology::new().context("Initialize hwloc topology")?;
    let depth = topology
        .depth_for_type(ObjectType::PU)
        .map_err(|error| anyhow!("Failed to get processing unit depth: {:?}", error))?;
    let cores = topology.objects_at_depth(depth).cycle();
    let numa = &config.global.numa;

    config.global.numa.bind(topology)?;

    let barrier = &Barrier::new(config.global.thread_count + 1);
    let mut map = I::new(&config.index);

    let (perf, threads) = thread::scope(|scope| -> anyhow::Result<_> {
        let workload = &config.workload;

        let mut perf_external = match (env::var("PERF_CTL_FIFO"), env::var("PERF_ACK_FIFO")) {
            (Ok(ctl), Ok(ack)) => Some(measure::perf::Sync::new(ctl, ack)?),
            _ => None,
        };
        let mut perf_internal = perf_external.is_none().then(|| measure::Perf::new());

        let coordinator = scope.spawn(move || -> anyhow::Result<_> {
            // Thread setup complete
            let _ = barrier.wait();

            if let Some(perf) = &mut perf_external {
                perf.enable()?;
            } else if let Some(perf) = &mut perf_internal {
                perf.start()
            };

            let _ = barrier.wait();

            // Threads complete
            let _ = barrier.wait();

            if let Some(perf) = &mut perf_external {
                perf.disable()?;
            } else if let Some(perf) = &mut perf_internal {
                return Ok(Some(perf.stop()));
            }

            Ok(None)
        });

        let threads = (0..config.global.thread_count)
            .zip(cores)
            .map(|(thread_id, core)| {
                let map = map.send();
                let key = &config.workload.key;

                scope.spawn(move || -> anyhow::Result<_> {
                    crate::THREAD_ID.set(thread_id);

                    let core_id = match numa {
                        crate::config::Numa::None | crate::config::Numa::Bind { node: 0 } => {
                            core.os_index().expect("No OS index for core")
                        }
                        // HACK: on our evaluation machine, this evenly distributes cores
                        // across NUMA nodes, rather than filling up node 0 first, then 1
                        crate::config::Numa::Interleave { .. } => thread_id,
                        crate::config::Numa::Bind { .. } => unimplemented!(),
                    };

                    log::debug!("Pin thread {thread_id} to core {core}");

                    topology
                        .bind_cpu(
                            core.cpuset().expect("No cpuset for core"),
                            CpuBindingFlags::THREAD
                                | CpuBindingFlags::STRICT
                                | CpuBindingFlags::NO_MEMORY_BINDING,
                        )
                        .context("Bind thread to CPU")?;

                    let mut loader =
                        workload.loader::<K>(key, config.global.thread_count, thread_id);
                    let mut runner = workload.runner::<K>(key);
                    let mut rng = SmallRng::seed_from_u64(thread_id as u64);
                    let mut map = map.pin();

                    if !workload.load {
                        while let Some(key) = loader.next_key() {
                            let checksum = K::Key::checksum(key);
                            let value = V::from_checksum(checksum);
                            map.insert(key, value);
                        }
                    }

                    if cfg!(feature = "stat") {
                        arctic::stat::start();
                    }

                    // Setup complete
                    let _ = barrier.wait();

                    if !workload.load && thread_id == 0 {
                        map.enable_membarrier();
                    }

                    // Perf enabled
                    let _ = barrier.wait();

                    let mut buffer = Vec::with_capacity(workload.ycsb.max_scan_length);

                    let start = Instant::now();

                    if config.workload.load {
                        while let Some(key) = loader.next_key() {
                            let checksum = K::Key::checksum(key);
                            let value = V::from_checksum(checksum);
                            map.insert(key, value);
                        }
                    } else {
                        for _ in 0..operation_count_per_thread {
                            let operation = runner.next_operation(&mut rng);
                            match operation {
                                ycsb::Operation::Read => {
                                    let (_, key) = runner.next_key_read(&mut rng);
                                    let _value = map.get(key);
                                    // if !I::IGNORE_GET {
                                    //     assert_eq!(value, Some(K::Key::checksum(key)));
                                    // }
                                }
                                ycsb::Operation::Update => {
                                    let (_, key) = runner.next_key_read(&mut rng);
                                    let checksum = K::Key::checksum(key);
                                    let value = V::from_checksum(checksum);
                                    let _old = map.update(key, value);
                                    // if !I::IGNORE_UPDATE {
                                    //     assert_eq!(old, Some(checksum));
                                    // }
                                }
                                ycsb::Operation::Scan => {
                                    let (_, key) = runner.next_key_read(&mut rng);
                                    let len = runner.next_scan_length(&mut rng);
                                    buffer.clear();
                                    map.scan(key, len, &mut buffer);
                                }
                                ycsb::Operation::Insert => {
                                    let (id, key) = runner.next_key_insert();
                                    let checksum = K::Key::checksum(key);
                                    let value = V::from_checksum(checksum);
                                    let _old = map.insert(key, value);
                                    // if !I::IGNORE_INSERT {
                                    //     assert_eq!(old, None);
                                    // }
                                    runner.acknowledge(id);
                                }
                                ycsb::Operation::ReadModifyWrite => todo!(),
                                ycsb::Operation::Delete => {
                                    let (_, key) = runner.next_key_read(&mut rng);
                                    let _ = map.remove(key);
                                }
                            }
                        }
                    }

                    let time = start.elapsed();

                    let _ = barrier.wait();

                    let index_report = map.report();

                    Ok(measure::Thread {
                        id: thread_id,
                        core: core_id,
                        time: time.as_nanos(),
                        operation_count: operation_count_per_thread as u64,
                        index: index_report,
                    })
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<anyhow::Result<Vec<_>>>()?;

        let perf = coordinator.join().unwrap()?;

        Ok((perf, threads))
    })?;

    let mimalloc = crate::measure::Mimalloc::new();
    let memory_key_value = map.memory_key_value();

    Ok(measure::Global {
        date,
        config,
        output: measure::Process {
            index: map.report(),
            perf,
            mimalloc,
            memory_key_value,
            thread: threads,
        },
    })
}
