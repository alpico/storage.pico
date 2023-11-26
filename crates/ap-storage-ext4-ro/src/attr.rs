//! File attributes for ext4.

use crate::{file::Ext4File, Error};
use ap_storage::attr;
use ap_util_slice_writer::*;

pub struct Attr<'a> {
    file: &'a Ext4File<'a>,
    offset: usize,
}

impl<'a> Attr<'a> {
    pub(crate) fn new(file: &'a Ext4File<'a>) -> Self {
        Self { file, offset: 0 }
    }
}

/// A macro to simplify the following code.
macro_rules! get {
    ($name:ident) => {{
        (stringify!($name), |buf, file| write!(buf, "{:x}", file.inode.$name()))
    }};
    ($name:ident, $param:expr) => {{
        (stringify!($name), $param)
    }};
}

/// The type for the attributes list.
type AttrType = (
    &'static str,
    fn(&mut SliceWriter, &Ext4File) -> Result<(), core::fmt::Error>,
);

/// A sorted list of file attributes.
const ATTRS: [AttrType; 16] = [
    get!(atime),
    get!(blocks, |b, f| write!(b, "{:x}", f.inode.blocks(f.fs.sb.block_size()))),
    get!(crtime),
    get!(ctime),
    get!(flags),
    get!(ftype, |b, f| write!(b, "{:?}", f.inode.ftype())),
    get!(generation),
    get!(gid),
    get!(id, |b, f| write!(b, "{:x}", f.nr)),
    get!(mode),
    get!(mtime),
    get!(nlinks),
    get!(size, |b, f| write!(b, "{:x}", f.inode.size(f.fs.sb.feature_incompat))),
    get!(uid),
    get!(version),
    get!(xattr),
];

impl<'a> attr::Attributes for Attr<'a> {
    fn next(&mut self, name: &mut [u8], value: &mut [u8]) -> Result<Option<attr::Entry>, Error> {
        let Some((n, f)) = ATTRS.get(self.offset) else {
            return Ok(None);
        };

        let mut name = SliceWriter(name, 0);
        let mut value = SliceWriter(value, 0);

        name.write_str(n)?;
        f(&mut value, self.file)?;

        self.offset += 1;
        Ok(Some(attr::Entry {
            name_len: name.1,
            value_len: value.1,
        }))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn attrs_is_sorted() {
        let mut iter = super::ATTRS.into_iter();
        let first = iter.next().unwrap();
        assert!(iter
            .try_fold(first, |prev, cur| { (prev < cur).then_some(cur) })
            .is_some())
    }
}
