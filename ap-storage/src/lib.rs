//! The alpico storage interfaces.
#![feature(error_in_core)]
#![no_std]

/// Offset in the underlying storage.
pub type Offset = u64;

/// Error when reading.
pub type Error = anyhow::Error;

mod read;
pub use read::*;
mod write;
pub mod directory;
pub mod file;


pub trait FileSystem {
    /// Return the root directory.
    fn root(&self) -> Result<impl file::File + '_, Error>;
}
