use core::cell::Cell;

pub(crate) mod benchmark;
pub mod config;
pub(crate) mod index;
pub(crate) mod measure;
pub(crate) mod workload;

thread_local! {
    pub(crate) static THREAD_ID: Cell<usize> = const { Cell::new(0) };
}

pub use benchmark::Benchmark;
pub use benchmark::run;
