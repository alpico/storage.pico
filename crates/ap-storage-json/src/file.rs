//! File interface for JsonFS.

use super::*;

/// File interface for JsonFS.
pub struct JsonFile<'a> {
    value: &'a serde_json::Value,
    pub(crate) id: u64,
}
impl<'a> JsonFile<'a> {
    /// Create a new JsonFile.
    pub fn new(value: &'a serde_json::Value, name: &str) -> Self {
        Self {
            value,
            id: name.as_ptr() as u64,
        }
    }

    /// Size in bytes.
    pub fn size(&self) -> u64 {
        let Ok(v) = serde_json::to_string(self.value) else {
            return 0;
        };
        v.as_bytes().len() as u64
    }
}

impl<'a> File for JsonFile<'a>
where
    Self: 'a,
{
    type AttrType<'c> = attr::Attr<'c> where Self: 'c;
    fn attr(&self) -> Self::AttrType<'_> {
        attr::Attr { file: self }
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
        let children = self.value.as_object().ok_or(msg2err!("not an object"))?;
        let child = children.keys().nth(offset as usize).ok_or(msg2err!("eof"))?;
        Ok(JsonFile::new(&children[child], child))
    }

    /// A more efficient lookup.
    fn lookup(&self, name: &[u8]) -> Result<Option<Self>, Error> {
        let name = core::str::from_utf8(name).map_err(|e| msg2err!(e))?;
        let children = self.value.as_object().ok_or(msg2err!("not an object"))?;
        let Some(value) = children.get(name) else {
            return Ok(None);
        };
        Ok(Some(JsonFile::new(value, name)))
    }
}

impl Read for JsonFile<'_> {
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let v = serde_json::to_string(self.value).map_err(|e| msg2err!(e))?;
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
