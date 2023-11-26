//! Support for directories.

use super::Error;
use crate::file::FileType;

/// Directory entry.
#[derive(Debug)]
pub struct DirEntry {
    /// The offset inside the parent. This is used to open the file relative to the parent.
    pub offset: u64,
    /// An unique ID of the referenced file.  Usefull for detecting hard-links.
    pub id: u64,
    /// The maximal length of the name this file has.
    pub nlen: usize,
    /// The file-type.
    pub typ: FileType,
}

/// Iterator over directories.
pub trait DirIterator {
    /// Return the next entry in this directory.
    ///
    /// Fills the name with upto `nlen` bytes.  If a shorter buffer
    /// is given the name is truncated.
    fn next(&mut self, name: &mut [u8]) -> Result<Option<DirEntry>, Error>;
}
