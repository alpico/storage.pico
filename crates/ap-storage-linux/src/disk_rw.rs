use super::*;
use ap_storage::Write;

/// A writeable Linux disk.
pub struct LinuxDiskRW(LinuxDisk);
impl LinuxDiskRW {
    pub fn new(filename: &str, offset: u64) -> Result<Self, Error> {
        Ok(Self(LinuxDisk {
            file: File::options().read(true).write(true).open(filename)?,
            offset,
        }))
    }
}

impl Read for LinuxDiskRW {
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        self.0.read_bytes(offset, buf)
    }
}

impl Write for LinuxDiskRW {
    fn write_bytes(&self, offset: Offset, buf: &[u8]) -> Result<usize, Error> {
        let res = unsafe {
            libc::pwrite(
                self.0.file.as_raw_fd(),
                buf.as_ptr() as *const libc::c_void,
                buf.len(),
                (self.0.offset + offset) as i64,
            )
        };
        if res == -1 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(res as usize)
    }

    fn discard(&self, offset: Offset, len: Offset) -> Result<Offset, Error> {
        let res = unsafe {
            libc::fallocate(
                self.0.file.as_raw_fd(),
                libc::FALLOC_FL_PUNCH_HOLE | libc::FALLOC_FL_KEEP_SIZE,
                offset as i64,
                len as i64,
            )
        };
        if res == -1 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(len)
    }
}
