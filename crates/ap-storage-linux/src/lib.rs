//! Linux specific storage interface.

use ap_storage::{Error, Offset, Read};
use std::fs::File;
use std::os::fd::AsRawFd;

mod disk_rw;
pub use disk_rw::LinuxDiskRW;

/// A disk backed by a file in Linux.
pub struct LinuxDisk {
    file: File,
    offset: u64,
}

impl LinuxDisk {
    /// Open a read-only disk at the given offset.
    pub fn new(filename: &str, offset: u64) -> Result<Self, Error> {
        Ok(Self {
            file: File::open(filename)?,
            offset,
        })
    }
}

impl Read for LinuxDisk {
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let res = unsafe {
            libc::pread(
                self.file.as_raw_fd(),
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                (self.offset + offset) as i64,
            )
        };
        if res == -1 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(res as usize)
    }
}
