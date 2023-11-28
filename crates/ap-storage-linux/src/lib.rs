//! Linux specific ap-storage implementations.

#![no_std]

use core::ffi::CStr;
mod disk_ro;
mod disk_rw;
pub use disk_ro::LinuxDiskRO;
pub use disk_rw::LinuxDiskRW;

/// Convert an libc error into a Result.
///
/// # Safety
/// - the underlying function is unsafe
pub unsafe fn check_error(res: isize) -> Result<isize, i32> {
    if res != -1 {
        return Ok(res);
    }
    Err(*libc::__errno_location())
}

/// Convert the input string into an CStr with a trailing nul.
///
/// The buffer is used as backing store and must be large enough.
pub fn str2cstr<'a>(input: &str, buf: &'a mut [u8]) -> Option<&'a CStr> {
    let src = input.as_bytes();
    if buf.len() <= src.len() {
        return None;
    }
    buf[..src.len()].copy_from_slice(src);
    buf[src.len()] = 0;
    CStr::from_bytes_until_nul(buf).ok()
}
