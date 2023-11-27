//! File attributes for ext4.
use crate::file::Ext4File;
use ap_storage::attr::{self, Attributes, Value, new_attr};
use ap_util_slice_writer::*;

new_attr!(BLOCKS, U64, "Number of blocks occupied.");
new_attr!(FLAGS, U64, "Inode flags.");
new_attr!(CTIME, I64, "Change time of file meta-data.");
new_attr!(GID, U64, "Group id.");
new_attr!(UID, U64, "User id.");
new_attr!(
    GENERATION,
    U64,
    "File generation number.  A random value to detect reused IDs."
);
new_attr!(MODE, U64, "File mode bits.");
new_attr!(NLINKS, U64, "Number of hard-links to this file.");
new_attr!(VERSION, U64, "Version number to detect file changes.");
new_attr!(XATTR, U64, "Block holding the extended attributes.");

pub struct Attr<'a> {
    pub(crate) file: &'a Ext4File<'a>,
}

impl<'a> IntoIterator for Attr<'a> {
    type Item = &'a &'a str;
    type IntoIter = core::slice::Iter<'a, &'a str>;
    fn into_iter(self) -> Self::IntoIter {
        [
            BLOCKS,
            CTIME,
            FLAGS,
            GENERATION,
            GID,
            MODE,
            NLINKS,
            UID,
            VERSION,
            XATTR,
            attr::ATIME,
            attr::BTIME,
            attr::FTYPE,
            attr::ID,
            attr::MTIME,
            attr::SIZE,
        ]
        .iter()
    }
}

impl<'a> Attributes<'a> for Attr<'a> {
    fn get(&self, name: &str, buf: &mut [u8]) -> Option<Value> {
        Some(match name {
            attr::FTYPE => {
                let mut value = SliceWriter(buf, 0);
                write!(value, "{:?}", self.file.ftype()).ok()?;
                Value::Raw(value.1)
            }
            BLOCKS => self.file.inode.blocks(self.file.fs.sb.block_size()).into(),
            CTIME => self.file.inode.ctime().into(),
            FLAGS => self.file.inode.flags().into(),
            GENERATION => self.file.inode.generation().into(),
            GID => self.file.inode.gid().into(),
            MODE => self.file.inode.mode().into(),
            NLINKS => self.file.inode.nlinks().into(),
            UID => self.file.inode.uid().into(),
            VERSION => self.file.inode.version().into(),
            XATTR => self.file.inode.xattr().into(),
            attr::ATIME => self.file.inode.atime().into(),
            attr::BTIME => self.file.inode.crtime().into(),
            attr::ID => self.file.nr.into(),
            attr::MTIME => self.file.inode.mtime().into(),
            attr::SIZE => self.file.inode.size(self.file.fs.sb.feature_incompat).into(),
            _ => return None,
        })
    }
}
