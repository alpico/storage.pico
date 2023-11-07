//! Support for files.

use crate::{Error, Offset};

/// Generic file-types.
#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Parent,
    SymLink,
    Unknown,
}



pub trait File {
    fn dir(&self) -> Option<impl crate::directory::Iterator>;
    fn open(&self, offset: Offset) -> Result<Self, Error> where Self:Sized;
    fn is_root(&self) -> bool;
    fn size(&self) -> Offset;
}
