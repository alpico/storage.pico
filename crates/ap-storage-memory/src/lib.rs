//! In-memory data structures.
//!
//! Provides caches and read-access to slices.
#![no_std]

use ap_storage::{Error, Offset, Read};
use core::cell::RefCell;

mod cache;
mod inline;
mod slice;
pub use slice::*;

/// A memory cache storing its data in an external slice.
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

/// A memory cache storing its data inside the object.
pub struct InlineCache<'a, const N: usize>(RefCell<inline::InlineCacheImpl<'a, N>>);

impl<'a, const N: usize> InlineCache<'a, N> {
    /// Create an inline memory cache.
    pub fn new(parent: &'a dyn Read) -> Self {
        Self(RefCell::new(inline::InlineCacheImpl::new(parent)))
    }
}

impl<const N: usize> Read for InlineCache<'_, N> {
    fn read_bytes(&self, ofs: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        self.0.borrow_mut().read_mut(ofs, buf)
    }
}
