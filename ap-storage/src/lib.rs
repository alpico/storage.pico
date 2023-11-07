//! The alpico storage interfaces.


/// Offset in the underlying storage.
pub type Offset = u64;

/// Error when reading.
pub type Error = anyhow::Error;

mod read;
pub use read::*;
mod write;
pub use write::*;
pub mod file;
pub mod directory;
