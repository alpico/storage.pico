//! Metadata for files.

use super::Offset;
pub type Time = i64;

/// Generic file-types.
#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    /// A plain file.
    File,
    /// A group of directories.
    Directory,
    /// The parent directory and the self-pointer.
    Parent,
    /// A symbolic link.
    SymLink,
    /// An unsupported entry.
    Unknown,
}


/// The minimal subset of metadata for all files.
pub struct MetaData {
    pub size: Offset,
    pub id: Offset,
    pub filetype: FileType,
    pub mtime: Time,
}
 
