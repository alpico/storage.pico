//! Write into a slice of bytes while truncating on overflow.

pub use core::fmt::Write;

/// Write into a slice of bytes while truncating on overflow.
pub struct SliceWriter<'a>(pub &'a mut [u8], pub usize);

impl Write for SliceWriter<'_> {
    fn write_str(&mut self, value: &str) -> Result<(), core::fmt::Error> {
        let b = value.as_bytes();
        if self.1 < self.0.len() {
            let n = core::cmp::min(self.0.len() - self.1, b.len());
            self.0[self.1..self.1 + n].copy_from_slice(&b[..n]);
        }
        self.1 += b.len();
        Ok(())
    }
}
