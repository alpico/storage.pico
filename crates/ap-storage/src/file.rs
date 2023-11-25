//! Support for files.

use crate::{attr::Attributes, directory::DirIterator, meta::FileType, meta::MetaData, Error, Offset};

/// A file trait.
pub trait File: crate::Read {
    type DirType<'c>: DirIterator
    where
        Self: 'c;
    type AttrType<'c>: Attributes
    where
        Self: 'c;
    /// Return a directory iterator.
    fn dir(&self) -> Option<Self::DirType<'_>>;

    /// Open children as offset from this directory.
    fn open(&self, offset: Offset) -> Result<Self, Error>
    where
        Self: Sized;

    /// Get the metadata for this file.
    fn meta(&self) -> MetaData;

    fn attr(&self) -> Self::AttrType<'_>;

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
                return Err(anyhow::anyhow!("file not found"));
            };
            res = x;
        }
        Ok(res)
    }
}
