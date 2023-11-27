use crate::file::PartitionFile;
use ap_storage::{
    attr::{Attributes, Value, FTYPE, ID, SIZE},
    new_attr,
};
use ap_util_slice_writer::*;

new_attr!(BOOT, Bool, "Boot flag.");
new_attr!(OFFSET, U64, "Offset of the partition in the underlying the disk.");
new_attr!(TYP, U64, "Type code.");

pub struct Attr<'a> {
    pub(crate) file: &'a PartitionFile<'a>,
}

impl<'a> IntoIterator for Attr<'a> {
    type Item = &'a &'a str;
    type IntoIter = core::slice::Iter<'a, &'a str>;
    fn into_iter(self) -> Self::IntoIter {
        [BOOT, FTYPE, ID, OFFSET, SIZE, TYP].iter()
    }
}

impl<'a> Attributes<'a> for Attr<'a> {
    fn get(&self, name: &str, buf: &mut [u8]) -> Option<Value> {
        Some(match name {
            FTYPE => {
                let mut value = SliceWriter(buf, 0);
                write!(value, "{:?}", self.file.ftype()).ok()?;
                Value::Raw(value.1)
            }
            BOOT => (self.file.drive & 0x80 != 0).into(),
            ID => self.file.id.into(),
            OFFSET => self.file.offset.into(),
            SIZE => self.file.len.into(),
            TYP => self.file.typ.into(),
            _ => return None,
        })
    }
}
