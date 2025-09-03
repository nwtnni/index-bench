use core::cell::Cell;

pub(crate) mod benchmark;
pub(crate) mod config;
pub(crate) mod index;
pub(crate) mod measure;

thread_local! {
    pub(crate) static THREAD_ID: Cell<usize> = const { Cell::new(0) };
}
