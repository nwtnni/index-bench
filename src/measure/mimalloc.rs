use std::ffi;

unsafe extern "C" {
    fn mi_stats_get_json() -> *mut ffi::c_char;
}

pub fn stats() -> serde_json::Value {
    let json = unsafe { ffi::CString::from_raw(mi_stats_get_json()) };
    let json = json.to_string_lossy();
    serde_json::from_str(&json).expect("Expected valid JSON from mimalloc")
}
