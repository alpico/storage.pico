use crate::file::JsonFile;
use ap_storage::attr::{Attributes, Value, ID, SIZE};

pub struct Attr<'a> {
    pub(crate) file: &'a JsonFile<'a>,
}

impl<'a> IntoIterator for Attr<'a> {
    type Item = &'a &'a str;
    type IntoIter = core::slice::Iter<'a, &'a str>;
    fn into_iter(self) -> Self::IntoIter {
        [ID, SIZE].iter()
    }
}

impl<'a> Attributes<'a> for Attr<'a> {
    fn get(&self, name: &str, _buf: &mut [u8]) -> Option<Value> {
        Some(match name {
            ID => self.file.id.into(),
            SIZE => self.file.size().into(),
            _ => return None,
        })
    }
}
