//! Using json object as file-system.
//!
//! - first example of a pseudo-filesystem
//! - usefull for small in-memory data

use ap_storage::{directory, file::File, file::FileType, msg2err, Error, FileSystem, Offset, Read};

mod attr;
mod dir;
mod file;

/// Making JSON availabe as a `ap_storage::FileSystem`.
pub struct JsonFS {
    root: serde_json::Value,
}

impl JsonFS {
    /// Create a new JsonFS by reading the whole JSON from the disk.
    pub fn new(disk: &dyn Read) -> Result<Self, Error> {
        // Fill the buffer.
        let mut data = vec![0; 4096];
        let mut ofs = 0;
        while let Ok(n) = disk.read_bytes(ofs as u64, &mut data[ofs..]) {
            if n == 0 {
                break;
            }
            ofs += n;
            if let Err(e) = serde_json::from_slice::<serde_json::Value>(&data[..]) {
                if !e.is_eof() {
                    return Err(msg2err!("invalid JSON"));
                }
            }
            data.resize(ofs + 4096, 0);
        }
        data.resize(ofs, 0);

        // convert to a Value
        let root: serde_json::Value = serde_json::from_slice(&data).map_err(|e| msg2err!(e))?;
        if !root.is_object() {
            return Err(msg2err!("not an object"));
        }
        Ok(Self { root })
    }
}

impl<'a> FileSystem<'a> for JsonFS {
    type FileType = file::JsonFile<'a>;
    fn root(&'a self) -> Result<Self::FileType, Error> {
        Ok(file::JsonFile::new(&self.root, "/"))
    }
}
