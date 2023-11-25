use super::*;
use ap_storage::attr::EmptyAttributes;
use ap_storage::meta::{FileType, MetaData};

pub struct JsonFile<'a> {
    value: &'a serde_json::Value,
    id: u64,
}
impl<'a> JsonFile<'a> {
    pub fn new(value: &'a serde_json::Value, name: &str) -> Self {
        Self {
            value,
            id: name.as_ptr() as u64,
        }
    }
}

impl<'a> File for JsonFile<'a>
where
    Self: 'a,
{
    type AttrType<'c> = EmptyAttributes where Self: 'c;
    fn attr(&self) -> Self::AttrType<'_> {
        EmptyAttributes
    }

    type DirType<'c> = crate::dir::JsonDir<'c> where Self: 'c;
    fn dir(&self) -> Option<Self::DirType<'_>> {
        let children = self.value.as_object()?;
        Some(crate::dir::JsonDir {
            keys: children.keys(),
            value: children,
            offset: 0,
        })
    }

    fn open(&self, offset: Offset) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let children = self.value.as_object().ok_or(anyhow::anyhow!("not an object"))?;
        let child = children.keys().nth(offset as usize).ok_or(anyhow::anyhow!("eof"))?;
        Ok(JsonFile::new(&children[child], child))
    }
    fn meta(&self) -> MetaData {
        MetaData {
            size: serde_json::to_string(self.value).map(|x| x.len()).unwrap_or_default() as Offset,
            id: self.id,
            mtime: 0,
            filetype: if self.value.is_object() {
                FileType::Directory
            } else {
                FileType::File
            },
        }
    }

    /// A more efficient lookup.
    fn lookup(&self, name: &[u8]) -> Result<Option<Self>, Error> {
        let name = core::str::from_utf8(name).map_err(Error::msg)?;
        let children = self.value.as_object().ok_or(anyhow::anyhow!("not an object"))?;
        let Some(value) = children.get(name) else {
            return Ok(None);
        };
        Ok(Some(JsonFile::new(value, name)))
    }
}

impl Read for JsonFile<'_> {
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let v = serde_json::to_string(self.value).map_err(Error::msg)?;
        let v = v.as_bytes();
        if offset >= v.len() as Offset {
            return Ok(0);
        }
        let ofs = offset as usize;
        let maxn = core::cmp::min(v.len() - ofs, buf.len());
        buf[..maxn].copy_from_slice(&v[ofs..ofs + maxn]);
        Ok(maxn)
    }
}
