use super::*;

pub struct JsonDir<'a> {
    pub keys: serde_json::map::Keys<'a>,
    pub value: &'a serde_json::Map<String, serde_json::Value>,
    pub offset: usize,
}

impl directory::DirIterator for JsonDir<'_> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<directory::DirEntry>, Error> {
        let Some(child) = self.keys.next() else {
            return Ok(None);
        };
        self.offset += 1;
        let maxn = core::cmp::min(name.len(), child.len());
        name[..maxn].copy_from_slice(child[..maxn].as_bytes());
        let typ = if self.value[child].is_object() {
            FileType::Directory
        } else {
            FileType::File
        };
        Ok(Some(directory::DirEntry {
            offset: self.offset as u64 - 1,
            nlen: child.len(),
            typ,
            id: child.as_ptr() as u64,
        }))
    }
}
