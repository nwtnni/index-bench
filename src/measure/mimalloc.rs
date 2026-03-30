use std::ffi;

use serde::Deserialize;
use serde::Serialize;

unsafe extern "C" {
    // https://github.com/migopp/mimalloc/blob/dev3/src/stats.c#L796
    fn mi_stats_get_json(buf_size: usize, buf: *mut ffi::c_char) -> *mut ffi::c_char;
}

// https://github.com/microsoft/mimalloc/blob/09a27098aa6e9286518bd9c74e6ffa7199c3f04e/include/mimalloc-stats.h
#[derive(Serialize, Deserialize)]
pub struct Mimalloc {
    pages: Count,
    reserved: Count,
    committed: Count,
    malloc_normal: Count,
    malloc_huge: Count,
    malloc_requested: Count,
}

impl Mimalloc {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        // Allocates buffer if `buf` is null.
        // https://github.com/migopp/mimalloc/blob/dev3/src/stats.c#L717
        let json = unsafe { ffi::CString::from_raw(mi_stats_get_json(0, std::ptr::null_mut())) };
        let json = json.to_string_lossy();
        serde_json::from_str(&json).expect("Expected valid JSON from mimalloc")
    }
}

#[derive(Serialize, Deserialize)]
pub struct Count {
    total: i64,
    peak: i64,
    current: i64,
}
