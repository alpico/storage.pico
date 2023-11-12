//! Support for files.

use crate::{directory::Iterator, Error, Offset};

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

/// A file trait.
pub trait File: crate::Read {
    type DirType<'c>: Iterator
    where
        Self: 'c;
    /// Return a directory iterator.
    fn dir(&self) -> Option<Self::DirType<'_>>;

    /// Open children as offset from this directory.
    fn open(&self, offset: Offset) -> Result<Self, Error>
    where
        Self: Sized;

    /// Get the size of this file.
    fn size(&self) -> Offset;

    /// The identity of the file.
    fn id(&self) -> u64;

    /// Lookup a single name and open the corresponding file.
    fn lookup(&self, name: &[u8]) -> Result<Option<Self>, Error>
    where
        Self: Sized,
    {
        let mut dir = self.dir().ok_or(anyhow::anyhow!("not a directory"))?;
        let mut buf = [0u8; 256];
        while let Some(entry) = dir.next(&mut buf)? {
            if entry.typ == FileType::Unknown {
                continue;
            }
            if &buf[..entry.nlen] == name {
                let res = self.open(entry.offset)?;
                return Ok(Some(res));
            }
        }
        Ok(None)
    }

    /// Lookup a whole path separated by slash
    fn lookup_path(self, path: &[u8]) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut res: Self = self;
        for name in path.split(|x| *x == b'/') {
            if name.is_empty() {
                continue;
            }
            let Some(x) = res.lookup(name)? else {
                return Err(anyhow::anyhow!("not found"));
            };
            res = x;
        }
        Ok(res)
    }
}
