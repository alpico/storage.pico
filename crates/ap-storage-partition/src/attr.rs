use crate::file::PartitionFile;
use ap_storage::attr::{AttrType, Attributes};
use ap_util_slice_writer::*;

pub struct Attr<'a> {
    pub(crate) file: &'a PartitionFile<'a>,
}

impl<'a> IntoIterator for Attr<'a> {
    type Item = &'a (AttrType, &'a str);
    type IntoIter = core::slice::Iter<'a, (AttrType, &'a str)>;
    fn into_iter(self) -> Self::IntoIter {
        [
            (AttrType::Raw, "ftype"),
            (AttrType::U64, "boot"),
            (AttrType::U64, "id"),
            (AttrType::U64, "offset"),
            (AttrType::U64, "size"),
            (AttrType::U64, "typ"),
        ]
        .iter()
    }
}

impl<'a> Attributes<'a> for Attr<'a> {
    fn get_raw(&mut self, name: &str, value: &mut [u8]) -> Option<usize> {
        let mut value = SliceWriter(value, 0);
        match name {
            "ftype" => write!(value, "{:?}", self.file.ftype()).ok()?,
            _ => return None,
        }
        Some(value.1)
    }
    fn get_u64(&mut self, name: &str) -> Option<u64> {
        Some(match name {
            "boot" => (self.file.drive & 0x80).into(),
            "id" => self.file.id,
            "offset" => self.file.offset,
            "size" => self.file.len,
            "typ" => self.file.typ.into(),
            _ => return None,
        })
    }
}
