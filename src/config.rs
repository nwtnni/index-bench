use serde::Deserialize;
use serde::Serialize;

mod numa;

pub(crate) use numa::Numa;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Global {
    /// Number of threads
    pub thread_count: usize,

    pub cargo: Cargo,

    pub numa: Numa,
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
