use super::*;
use ap_storage::{Error, Offset, Read, msg2err};

/// A disk backed by a file in Linux.
pub struct LinuxDiskRO {
    pub(crate) fd: i32,
    pub(crate) offset: u64,
}

impl LinuxDiskRO {
    /// Open a read-only disk at the given offset.
    pub fn new(filename: &str, offset: u64) -> Result<Self, Error> {
        let mut buf = [0u8; libc::PATH_MAX as usize];
        let filename = str2cstr(filename, &mut buf).ok_or(msg2err!("invalid filename"))?;
        let fd = unsafe {
            check_error(libc::open(filename.as_ptr(), libc::O_RDONLY) as isize)
                .map_err(|e| msg2err!("open").context(e))? as i32
        };
        Ok(Self { fd, offset })
    }
}

impl Read for LinuxDiskRO {
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let res = unsafe {
            check_error(libc::pread(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                (self.offset + offset) as i64,
            ))
            .map_err(|e| msg2err!("pread").context(e))?
        };
        Ok(res as usize)
    }
}

/// Close the file when the object drops.
impl Drop for LinuxDiskRO {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
        self.fd = -1;
    }
}
