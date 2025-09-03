use anyhow::Context as _;
use hwlocality::Topology;
use hwlocality::memory::binding::MemoryBindingFlags;
use hwlocality::memory::binding::MemoryBindingPolicy;
use hwlocality::memory::nodeset::NodeSet;
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
                MemoryBindingFlags::PROCESS
                    | MemoryBindingFlags::STRICT
                    | MemoryBindingFlags::MIGRATE
                    | MemoryBindingFlags::NO_CPU_BINDING,
            )
            .context("Bind process to NUMA node")
    }
}
