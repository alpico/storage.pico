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
pub mod meta;
mod write;
pub use write::*;

/// Hierarchical filesystem.
pub trait FileSystem<'a> {
    /// The type to represent files.
    type FileType: file::File;
    /// Return the root directory.
    fn root(&'a self) -> Result<Self::FileType, Error>;
}
