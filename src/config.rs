use serde::Deserialize;
use serde::Serialize;

mod numa;

pub use numa::Numa;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Global {
    /// Number of threads
    pub thread_count: usize,

    cargo: Cargo,

    pub numa: Numa,
}

impl Global {
    pub fn new(thread_count: usize, numa: Numa) -> Self {
        Self {
            thread_count,
            cargo: Cargo::default(),
            numa,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cargo {
    release: bool,
}

impl Default for Cargo {
    fn default() -> Self {
        Self {
            release: !cfg!(debug_assertions),
        }
    }
}
