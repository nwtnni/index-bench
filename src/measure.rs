pub(crate) mod perf;
mod resource;

pub(crate) use perf::Perf;
pub(crate) use resource::Resource;

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
    pub thread: Vec<Thread>,
}

#[derive(Deserialize, Serialize)]
pub struct Thread {
    pub id: usize,
    pub core: usize,
    pub time: u128,
    pub operation_count: u64,
    pub resource: Resource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perf: Option<perf::Report>,
    pub index: serde_json::Value,
}
