//! The alpico storage interfaces.
#![allow(async_fn_in_trait)]
#![no_std]

/// Offset in the underlying storage.
pub type Offset = u64;

/// Error when reading.
pub type Error = anyhow::Error;

mod read;
pub use read::*;
mod write;
pub use write::*;
