//! Memory mapped disk.

use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::{
    fs::OpenOptions,
    io::{Seek, SeekFrom},
};

use ap_storage::{Error, Read};

#[derive(Clone)]
pub struct MemDisk<'a> {
    data: &'a [u8],
}

impl<'a> MemDisk<'a> {
    /// Create a MemDisk by memory-mapping the file.
    pub fn new(filename: &str, direct: bool) -> Result<Self, Error> {
        let mut fd = OpenOptions::new()
            .read(true)
            .custom_flags(if direct { libc::O_DIRECT } else { 0 })
            .open(filename)
            .map_err(anyhow::Error::msg)?;

        let length = fd.seek(SeekFrom::End(0))?;

        // get a memory mapping
        let ptr = unsafe {
            libc::mmap(
                0 as _,
                length as _,
                libc::PROT_READ,
                libc::MAP_SHARED,
                fd.as_raw_fd(),
                0,
            )
        };
        assert_ne!(libc::MAP_FAILED, ptr);
        let data = unsafe { core::slice::from_raw_parts(ptr as _, length as _) };

        // tell the kernel to no read-ahead
        let x = unsafe { libc::madvise(ptr, length as _, libc::MADV_RANDOM) };
        assert_eq!(x, 0);
        Ok(Self { data })
    }
}

impl Drop for MemDisk<'_> {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.data.as_ptr() as *mut libc::c_void, self.data.len()) };
        self.data = &[];
    }
}

impl Read for MemDisk<'_> {
    fn read_bytes(&self, ofs: u64, buf: &mut [u8]) -> Result<usize, Error> {
        if ofs >= self.data.len() as u64 {
            return Ok(0);
        }
        let ofs = ofs as usize;
        let n = core::cmp::min(self.data.len() - ofs, buf.len());
        buf[..n].copy_from_slice(&self.data[ofs..ofs + n]);
        Ok(n)
    }
}
