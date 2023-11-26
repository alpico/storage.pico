use crate::file::JsonFile;
use ap_storage::attr::{AttrType, Attributes};

pub struct Attr<'a> {
    pub(crate) file: &'a JsonFile<'a>,
}

impl<'a> IntoIterator for Attr<'a> {
    type Item = &'a (AttrType, &'a str);
    type IntoIter = core::slice::Iter<'a, (AttrType, &'a str)>;
    fn into_iter(self) -> Self::IntoIter {
        [(AttrType::U64, "id")].iter()
    }
}

impl<'a> Attributes<'a> for Attr<'a> {
    fn get_u64(&mut self, name: &str) -> Option<u64> {
        Some(match name {
            "id" => self.file.id,
            _ => return None,
        })
    }
}
