use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "policy", rename_all = "snake_case")]
pub enum Numa {
    Bind { node: usize },
    Interleave { nodes: Vec<usize> },
}

impl Numa {
    pub fn set_mempolicy(&self) -> anyhow::Result<()> {
        // Call syscall to avoid external C dependency on `libnuma`.
        //
        // https://man7.org/linux/man-pages/man2/set_mempolicy.2.html
        unsafe fn set_mempolicy_syscall(
            mode: libc::c_int,
            mask: *const libc::c_ulong,
            maxnode: libc::c_ulong,
        ) -> i64 {
            unsafe { libc::syscall(libc::SYS_set_mempolicy, mode, mask, maxnode) }
        }

        let (mode, mask) = self.to_mode_mask();

        unsafe {
            match set_mempolicy_syscall(mode, &mask, 64) {
                -1 => Err(anyhow!("Failed to call set_mempolicy with {:?}", self)),
                _ => Ok(()),
            }
        }
    }

    fn to_mode_mask(&self) -> (libc::c_int, libc::c_ulong) {
        let (mode, mask) = match self {
            Numa::Bind { node } => (libc::MPOL_BIND, 1u64 << node),
            Numa::Interleave { nodes } => (
                libc::MPOL_INTERLEAVE,
                nodes.iter().map(|node| 1u64 << node).fold(0, |l, r| l | r),
            ),
        };

        (mode | libc::MPOL_F_STATIC_NODES, mask)
    }
}
