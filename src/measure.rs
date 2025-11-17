#[cfg(feature = "mimalloc")]
pub(crate) mod mimalloc;
pub(crate) mod perf;
pub mod resource;

#[cfg(feature = "mimalloc")]
pub(crate) use mimalloc::Mimalloc;
pub(crate) use perf::Perf;
pub use resource::Resource;

use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct Global {
    pub date: u128,
    pub config: crate::Config,
    pub output: Process,
}

#[derive(Deserialize, Serialize)]
pub struct Process {
    pub index: serde_json::Value,
    #[cfg(feature = "mimalloc")]
    pub mimalloc: Mimalloc,
    pub thread: Vec<Thread>,
}

#[derive(Deserialize, Serialize)]
pub struct Thread {
    pub id: usize,
    pub core: usize,
    pub time: u128,
    pub operation_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perf: Option<perf::Report>,
    pub index: serde_json::Value,
}
