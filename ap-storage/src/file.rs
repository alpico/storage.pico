//! Support for files.

/// Generic file-types.
#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Parent,
    SymLink,
    Unknown,
}


