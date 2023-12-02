use super::*;
use ap_storage::{msg2err, Error, Offset, Read, Write};

/// A writeable Linux disk.
pub struct LinuxDiskRW(LinuxDiskRO);
impl LinuxDiskRW {
    /// Use a file at a certain offset as a Linux disk.
    pub fn new(filename: &str, offset: u64) -> Result<Self, Error> {
        let mut buf = [0u8; libc::PATH_MAX as usize];
        let filename = str2cstr(filename, &mut buf).ok_or(msg2err!("invalid filename"))?;
        let fd = unsafe {
            check_error(libc::open(filename.as_ptr(), libc::O_RDWR) as isize)
                .map_err(|e| msg2err!("could not open file").context(e))? as i32
        };

        Ok(Self(LinuxDiskRO { fd, offset }))
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
            check_error(libc::pwrite(
                self.0.fd,
                buf.as_ptr() as *const libc::c_void,
                buf.len(),
                (self.0.offset + offset) as i64,
            ))
            .map_err(|e| msg2err!("pwrite").context(e))? as i32
        };
        Ok(res as usize)
    }

    fn discard(&self, offset: Offset, len: Offset) -> Result<Offset, Error> {
        unsafe {
            check_error(libc::fallocate(
                self.0.fd,
                libc::FALLOC_FL_PUNCH_HOLE | libc::FALLOC_FL_KEEP_SIZE,
                (self.0.offset + offset) as i64,
                len as i64,
            ) as isize)
            .map_err(|e| msg2err!("discard").context(e))? as i32
        };
        Ok(len)
    }
}
