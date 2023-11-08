//! The alpico storage interfaces.
#![feature(error_in_core)]
#![no_std]

/// Offset in the underlying storage.
pub type Offset = u64;

/// Error when reading.
pub type Error = anyhow::Error;

mod read;
pub use read::*;
pub mod directory;
pub mod file;
mod write;

pub trait FileSystem<'a> {
    type FileType;
    /// Return the root directory.
    fn root(&'a self) -> Result<Self::FileType, Error>;
}
