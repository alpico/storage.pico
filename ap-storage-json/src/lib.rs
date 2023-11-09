use ap_storage::{
    directory,
    file::{File, FileType},
    Error, FileSystem, Offset, Read,
};

mod dir;
mod file;

pub struct JsonFS {
    root: serde_json::Value,
}

impl JsonFS {
    pub fn new(disk: &impl Read) -> Result<Self, Error> {
        // Fill the buffer.
        let mut data = vec![0; 4096];
        let mut ofs = 0;
        while let Ok(n) = disk.read_bytes(ofs as u64, &mut data[ofs..]) {
            if n == 0 {
                break;
            }
            ofs += n;
            data.resize(ofs + 4096, 0);
        }
        data.resize(ofs, 0);

        // convert to a Value
        let root: serde_json::Value = serde_json::from_slice(&data).map_err(Error::msg)?;
        if !root.is_object() {
            return Err(anyhow::anyhow!("not an object"));
        }
        Ok(Self { root })
    }
}

impl<'a> FileSystem<'a> for JsonFS {
    type FileType = file::JsonFile<'a>;
    fn root(&'a self) -> Result<Self::FileType, Error> {
        Ok(file::JsonFile { value: &self.root })
    }
}
