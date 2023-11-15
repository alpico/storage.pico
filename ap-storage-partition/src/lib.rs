//! MBR Partition support.
#![no_std]

use anyhow;
use ap_storage::{Error, FileSystem, Offset, Read, ReadExt, file::{File, FileType}, directory::{Iterator, Item}};
use core::fmt::Write;


#[derive(Clone)]
pub struct PartitionFS<'a> {
    disk: &'a dyn Read,
    len: u64,
}

/// A single partition - on-disk format.
#[repr(C)]
#[derive(Clone)]
pub struct Partition {
    pub drive: u8,
    pub _chs_start: [u8; 3],
    pub typ: u8,
    pub _chs_end: [u8; 3],
    pub lba: u32,
    pub size: u32,
}


impl<'a> PartitionFS<'a> {
    /// Mount the filesystem.
    pub fn new(disk: &'a dyn Read) -> Result<Self, Error> {
        let buf: [u8; 512] = disk.read_object(0)?;
        if buf[0x1fe] != 0x55 || buf[0x1ff] != 0xaa {
            return Err(anyhow::anyhow!("not an MBR"));
        }
        // find the maximum length all partitions occupy
        let primary: [Partition; 4] = unsafe  { core::ptr::read_unaligned(buf.as_ptr().add(0x1be).cast()) };
        let len = primary.iter().map(|x| x.lba + x.size).fold(0, |x,y| core::cmp::max(x,y));
        Ok(Self { disk, len: (len as u64) * 512 })
    }
}



impl<'a> FileSystem<'a> for PartitionFS<'a> {
    type FileType = PartitionFile<'a>;
    fn root(&'a self) -> Result<<Self as FileSystem<'a>>::FileType, anyhow::Error> {
        Ok(PartitionFile { fs: self, offset: 0, len: self.len } )
    }
}

pub struct PartitionFile<'a> {
    fs: &'a PartitionFS<'a>,
    // the absolute offset on the disk
    offset: Offset,
    // the len in bytes
    len: Offset,
}

impl Read for PartitionFile<'_> {
    fn read_bytes(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Error> {
        if offset >= self.offset + self.len {
            return Ok(0)
        }
        let maxn = core::cmp::min(self.offset + self.len - offset, buf.len() as u64) as usize;
        self.fs.disk.read_bytes(self.offset + offset, &mut buf[..maxn])
    }
}
 
pub struct PartitionDir<'a>{
    file: &'a PartitionFile<'a>,
    pos: usize,
}

enum Alias {
    Raw,
    Boot,
    Type,
    Max,
}



impl Iterator for PartitionDir<'_> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<Item>, Error> {
        if self.pos >= 4 * Alias::Max as usize {
            return Ok(None);
        }

        let part = self.pos / Alias::Max as usize;
        let partition: Partition = self.file.fs.disk.read_object(self.file.offset + 0x1be + part as u64 * 0x10 )?;
        let mut writer = SliceWriter(name, 0);
        match self.pos % Alias::Max as usize {
            x if x == Alias::Raw as usize => { core::write!(&mut writer, "raw-{}", part).unwrap() },
            x if x == Alias::Boot as usize => {
                if partition.drive & 0x80 != 0 {
                    core::write!(&mut writer, "boot-{}", part).unwrap();
                }},
            x if x == Alias::Type as usize => { core::write!(&mut writer, "type-{:02x}-{}", partition.typ, part).unwrap(); },
            _ => { unreachable!() }
        };


        let typ = if partition.typ == 0 || partition.size == 0 || writer.1 == 0 {
            FileType::Unknown
        }
        else {
            if let Ok(child) = self.file.open(self.pos as u64) {
                if child.is_dir() {
                    FileType::Directory
                }
                else {
                    FileType::File
                }
            } else {
                FileType::Unknown
            }
        };
        self.pos += 1;
        Ok(Some(Item {
            nlen: writer.1,
            id: part as u64,
            offset: self.pos as u64 - 1,
            typ,
        }))
    }
}

/// Write into a slice of bytes while truncating on overflow.
struct SliceWriter<'a>(pub &'a mut [u8], pub usize);
impl Write for SliceWriter<'_> {
    fn write_str(&mut self, value: &str) -> Result<(), core::fmt::Error> {
        let b = value.as_bytes();
        if self.1 < self.0.len() {
            let n = core::cmp::min(self.0.len() - self.1, b.len());
            self.0[self.1..self.1 + n].copy_from_slice(&b[..n]);
        }
        self.1 += b.len();
        Ok(())
    }
}

impl PartitionFile<'_> {
    fn is_dir(&self) -> bool {
        let buf: [u8; 2] = self.fs.disk.read_object(self.offset + 0x1fe).unwrap_or([0, 0]);
        buf[0] == 0x55 && buf[1] == 0xaa
    }
}


impl File for PartitionFile<'_> {
    type DirType<'a> = PartitionDir<'a> where Self: 'a;
    fn dir(&self) -> Option<<Self as ap_storage::file::File>::DirType<'_>> {
        if self.is_dir() {
            return Some(PartitionDir{ file: self, pos: 0 })
        }
        None
    }
    fn open(&self, offset: u64) -> Result<Self, anyhow::Error> {
        let part = offset / Alias::Max as u64;
        if part > 4 {
            return Err(anyhow::anyhow!("invalid number"));
        }
        let partition: Partition = self.fs.disk.read_object(self.offset + 0x1be + part * 0x10 )?;
        let offset = self.offset + (partition.lba as u64) * 512;
        let len = if offset >= self.offset + self.len {
            0
        } else {
            core::cmp::min(self.offset + self.len - offset, (partition.size as u64) * 512)
        };

        Ok(PartitionFile { fs: self.fs, offset, len } )
    }
    fn size(&self) -> u64 { self.len }
    fn id(&self) -> u64 { self.offset }
}
