//! Disk usage on the root filesystem via statvfs("/"). Returns percent used.
//!
//! We use the libc statvfs syscall directly rather than a crate, since pulling
//! in nix or sysinfo for one syscall isn't worth the dep. The structure layout
//! is stable on Linux glibc and musl.

use anyhow::Result;

use super::{Reading, SensorId};

#[repr(C)]
#[derive(Default)]
struct Statvfs {
    f_bsize: u64,
    f_frsize: u64,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_favail: u64,
    f_fsid: u64,
    f_flag: u64,
    f_namemax: u64,
    __spare: [i32; 6],
}

extern "C" {
    fn statvfs(path: *const i8, buf: *mut Statvfs) -> i32;
}

pub async fn read() -> Result<Reading> {
    let used_pct = tokio::task::spawn_blocking(|| -> Result<f64> {
        let path = b"/\0";
        let mut buf = Statvfs::default();
        let rc = unsafe { statvfs(path.as_ptr() as *const i8, &mut buf as *mut Statvfs) };
        if rc != 0 {
            anyhow::bail!("statvfs failed: rc={rc}");
        }
        let total = buf.f_blocks;
        let avail = buf.f_bavail;
        if total == 0 {
            return Ok(0.0);
        }
        Ok(100.0 * (1.0 - (avail as f64 / total as f64)))
    })
    .await??;
    Ok(Reading {
        sensor: SensorId::Disk,
        value: used_pct.clamp(0.0, 100.0),
        context: None,
    })
}
