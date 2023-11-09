//! Support for files.

use crate::{directory::Iterator, Error, Offset};

/// Generic file-types.
#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Parent,
    SymLink,
    Unknown,
}

pub trait File: crate::Read {
    fn dir(&self) -> Option<impl Iterator>;
    fn open(&self, offset: Offset) -> Result<Self, Error>
    where
        Self: Sized;
    fn size(&self) -> Offset;
}
