use anyhow::Context as _;
use hwlocality::Topology;
use hwlocality::memory::binding::MemoryBindingFlags;
use hwlocality::memory::binding::MemoryBindingPolicy;
use hwlocality::memory::nodeset::NodeSet;
use hwlocality::topology::support;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "policy", rename_all = "snake_case")]
pub enum Numa {
    None,
    Bind { node: usize },
    Interleave { nodes: Vec<usize> },
}

impl Numa {
    pub(crate) fn bind(&self, topology: &Topology) -> anyhow::Result<()> {
        let (nodeset, policy) = match self {
            Numa::None => return Ok(()),
            Numa::Bind { node } => (
                topology
                    .node_with_os_index(*node)
                    .expect("No NUMA node")
                    .nodeset()
                    .expect("No nodeset")
                    .clone_target(),
                MemoryBindingPolicy::Bind,
            ),
            Numa::Interleave { nodes } => (
                nodes
                    .iter()
                    .map(|node| topology.node_with_os_index(*node).expect("No NUMA node"))
                    .map(|node| node.nodeset().expect("No nodeset"))
                    .fold(NodeSet::new(), |set, node| set | node),
                MemoryBindingPolicy::Interleave,
            ),
        };

        topology
            .bind_memory(
                &nodeset,
                policy,
                // hwloc doesn't support `MemoryBindingFlags::PROCESS`, but
                // thread still calls `set_mempolicy` for Linux, which should
                // be inherited by new threads.
                //
                // See:
                // - https://github.com/open-mpi/hwloc/blob/c124d197dfae0d8382128e610c542f4a84393bb9/hwloc/topology-linux.c#L1966-L2038
                // - https://www.kernel.org/doc/Documentation/vm/numa_memory_policy.txt
                //
                // > In a multi-threaded task, task policies apply only to the thread
                // > [Linux kernel task] that installs the policy and any threads
                // > subsequently created by that thread.
                if topology.supports(
                    support::FeatureSupport::memory_binding,
                    support::MemoryBindingSupport::set_current_process,
                ) {
                    MemoryBindingFlags::PROCESS
                } else {
                    MemoryBindingFlags::THREAD
                } | MemoryBindingFlags::STRICT
                    | MemoryBindingFlags::MIGRATE
                    | MemoryBindingFlags::NO_CPU_BINDING,
            )
            .context("Bind process to NUMA node")
    }
}
