//! In-memory data structures.
#![no_std]

use ap_storage::{Error, Offset, Read};
use core::cell::RefCell;

mod cache;
mod slice;
pub use slice::*;

/// A memory cache.
pub struct MemoryCache<'a>(RefCell<cache::MemoryCacheImpl<'a>>);

impl<'a> MemoryCache<'a> {
    /// Create a new cache by using data as backing store.
    pub fn new(data: &'a mut [u8], parent: &'a dyn Read) -> Self {
        Self(RefCell::new(cache::MemoryCacheImpl::new(data, parent)))
    }
}

impl Read for MemoryCache<'_> {
    fn read_bytes(&self, ofs: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        self.0.borrow_mut().read_mut(ofs, buf)
    }
}
