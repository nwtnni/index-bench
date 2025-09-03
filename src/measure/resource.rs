use core::mem::MaybeUninit;
use core::ops::Sub;
use std::io;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize)]
pub struct Resource {
    max_rss: u64,
    user_time: u128,
    system_time: u128,
    minor_fault_count: u64,
    major_fault_count: u64,
    voluntary_context_switch_count: u64,
    involuntary_context_switch_count: u64,
}

impl Sub for Resource {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            max_rss: self.max_rss.max(rhs.max_rss),
            user_time: self.user_time.saturating_sub(rhs.user_time),
            system_time: self.system_time.saturating_sub(rhs.system_time),
            minor_fault_count: self.minor_fault_count.saturating_sub(rhs.minor_fault_count),
            major_fault_count: self.major_fault_count.saturating_sub(rhs.major_fault_count),
            voluntary_context_switch_count: self
                .voluntary_context_switch_count
                .saturating_sub(rhs.voluntary_context_switch_count),
            involuntary_context_switch_count: self
                .involuntary_context_switch_count
                .saturating_sub(rhs.involuntary_context_switch_count),
        }
    }
}

impl Resource {
    pub(crate) fn new() -> io::Result<Self> {
        let rusage = unsafe {
            let mut rusage = MaybeUninit::<libc::rusage>::zeroed();
            match libc::getrusage(libc::RUSAGE_THREAD, rusage.as_mut_ptr()) {
                0 => rusage.assume_init(),
                _ => return Err(io::Error::last_os_error()),
            }
        };

        Ok(Self {
            max_rss: rusage.ru_maxrss as u64 * 2u64.pow(10),
            user_time: Self::timeval_as_nanos(rusage.ru_utime),
            system_time: Self::timeval_as_nanos(rusage.ru_stime),
            minor_fault_count: rusage.ru_minflt as u64,
            major_fault_count: rusage.ru_majflt as u64,
            voluntary_context_switch_count: rusage.ru_nvcsw as u64,
            involuntary_context_switch_count: rusage.ru_nivcsw as u64,
        })
    }

    fn timeval_as_nanos(time: libc::timeval) -> u128 {
        let s = time.tv_sec as u128 * 10u128.pow(9);
        let us = time.tv_usec as u128 * 10u128.pow(6);
        s + us
    }
}
