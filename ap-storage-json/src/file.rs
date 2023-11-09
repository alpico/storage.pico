use super::*;

pub struct JsonFile<'a> {
    pub value: &'a serde_json::Value,
}

impl File for JsonFile<'_> {
    fn dir(&self) -> Option<impl directory::Iterator> {
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
        let children = self
            .value
            .as_object()
            .ok_or(anyhow::anyhow!("not an object"))?;
        let child = children
            .keys()
            .nth(offset as usize)
            .ok_or(anyhow::anyhow!("eof"))?;
        Ok(JsonFile {
            value: &children[child],
        })
    }
    fn size(&self) -> Offset {
        serde_json::to_string(self.value)
            .map(|x| x.len())
            .unwrap_or_default() as Offset
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
        buf.copy_from_slice(&v[ofs..ofs + maxn]);
        Ok(maxn)
    }
}
