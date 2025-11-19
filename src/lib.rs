#[global_allocator]
static MIMALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

use core::cell::Cell;

pub(crate) mod benchmark;
pub mod config;
pub mod index;
pub mod measure;
pub mod workload;

pub(crate) use index::Index;

thread_local! {
    pub(crate) static THREAD_ID: Cell<usize> = const { Cell::new(0) };
}

pub use benchmark::run;
use cartesian::Cartesian;
use serde::Deserialize;
use serde::Serialize;

#[derive(Cartesian)]
#[rustfmt::skip]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub index: index::Config,
    #[cartesian(flatten)]
    global: config::Global,
    #[cartesian(flatten)]
    pub workload: workload::Config,
}
