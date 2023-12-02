//! The alpico storage interfaces.
#![no_std]

/// Offset in the underlying storage.
pub type Offset = u64;

/// Error when reading.
pub type Error = anyhow::Error;

pub mod attr;
pub mod directory;
pub mod file;
mod read;
mod write;

pub use read::*;
pub use write::*;

/// Hierarchical filesystem.
pub trait FileSystem<'a> {
    /// The type to represent files.
    type FileType: file::File;
    /// Return the root directory.
    fn root(&'a self) -> Result<Self::FileType, Error>;
}

/// Check for errors including the location as context.
#[macro_export]
macro_rules! check {
    ($v: expr) => { $v.map_err(|e| e.context($crate::ErrorCtx((file!(), line!()))))? }
}



/// Convert into an error type including the context.
#[macro_export]
macro_rules! msg2err {
    ($v: expr) => { Error::msg($v).context($crate::ErrorCtx((file!(), line!()))) }
}

/// A container for file! and line! Error context
pub struct ErrorCtx(pub (&'static str, u32));
impl core::fmt::Display for ErrorCtx {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(fmt, "{}:{}", self.0.0, self.0.1)
    }
}
