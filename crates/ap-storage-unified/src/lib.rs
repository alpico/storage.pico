//! Unifiy all filesystems by wrapping them into an enum.

#![no_std]

// This is only wrapper code. So no docs required.
#![allow(missing_docs)]

use ap_storage::directory::{DirEntry, DirIterator};
use ap_storage::{
    attr::{Attributes, Value},
    file::File,
    Error, FileSystem, Read,
};
use ap_storage_ext4_ro::Ext4Fs;
use ap_storage_json::JsonFS;
use ap_storage_partition::PartitionFS;
use ap_storage_vfat_ro::VFatFS;

#[allow(clippy::large_enum_variant)]
pub enum UnifiedFs<'a> {
    Ext4(Ext4Fs<'a>),
    Json(JsonFS),
    Vfat(VFatFS<'a>),
    Partition(PartitionFS<'a>),
}

impl<'a> UnifiedFs<'a> {
    /// try to mount all file-systems.
    pub fn new(disk: &'a dyn Read) -> Option<Self> {
        if let Ok(f) = Ext4Fs::new(disk, false) {
            return Some(Self::Ext4(f));
        }
        if let Ok(f) = JsonFS::new(disk) {
            return Some(Self::Json(f));
        }
        if let Ok(f) = VFatFS::new(disk, Default::default()) {
            return Some(Self::Vfat(f));
        }
        if let Ok(f) = PartitionFS::new(disk) {
            return Some(Self::Partition(f));
        }
        None
    }
}

impl<'a> FileSystem<'a> for UnifiedFs<'a> {
    type FileType = UnifiedFile<'a>;
    fn root(&'a self) -> Result<<Self as FileSystem<'a>>::FileType, Error> {
        Ok(match self {
            UnifiedFs::Ext4(f) => UnifiedFile::Ext4(f.root()?),
            UnifiedFs::Json(f) => UnifiedFile::Json(f.root()?),
            UnifiedFs::Vfat(f) => UnifiedFile::Vfat(f.root()?),
            UnifiedFs::Partition(f) => UnifiedFile::Partition(f.root()?),
        })
    }
}

pub enum UnifiedFile<'a> {
    Ext4(<Ext4Fs<'a> as FileSystem<'a>>::FileType),
    Json(<JsonFS as FileSystem<'a>>::FileType),
    Vfat(<VFatFS<'a> as FileSystem<'a>>::FileType),
    Partition(<PartitionFS<'a> as FileSystem<'a>>::FileType),
}

impl<'a> File for UnifiedFile<'a> {
    type AttrType<'c> = UnifiedAttr<'c> where Self: 'c;
    fn attr(&self) -> Self::AttrType<'_> {
        match self {
            UnifiedFile::Ext4(f) => UnifiedAttr::Ext4(f.attr()),
            UnifiedFile::Json(f) => UnifiedAttr::Json(f.attr()),
            UnifiedFile::Vfat(f) => UnifiedAttr::Vfat(f.attr()),
            UnifiedFile::Partition(f) => UnifiedAttr::Partition(f.attr()),
        }
    }
    type DirType<'c> = UnifiedDir<'c> where Self: 'c;
    fn dir(&self) -> Option<Self::DirType<'_>> {
        Some(match self {
            UnifiedFile::Ext4(f) => UnifiedDir::Ext4(f.dir()?),
            UnifiedFile::Json(f) => UnifiedDir::Json(f.dir()?),
            UnifiedFile::Vfat(f) => UnifiedDir::Vfat(f.dir()?),
            UnifiedFile::Partition(f) => UnifiedDir::Partition(f.dir()?),
        })
    }
    fn open(&self, offset: u64) -> Result<Self, Error> {
        Ok(match self {
            UnifiedFile::Ext4(f) => UnifiedFile::Ext4(f.open(offset)?),
            UnifiedFile::Json(f) => UnifiedFile::Json(f.open(offset)?),
            UnifiedFile::Vfat(f) => UnifiedFile::Vfat(f.open(offset)?),
            UnifiedFile::Partition(f) => UnifiedFile::Partition(f.open(offset)?),
        })
    }
}

impl<'a> Read for UnifiedFile<'a> {
    fn read_bytes(&self, ofs: u64, buf: &mut [u8]) -> Result<usize, Error> {
        match self {
            UnifiedFile::Ext4(f) => f.read_bytes(ofs, buf),
            UnifiedFile::Json(f) => f.read_bytes(ofs, buf),
            UnifiedFile::Vfat(f) => f.read_bytes(ofs, buf),
            UnifiedFile::Partition(f) => f.read_bytes(ofs, buf),
        }
    }
}

pub enum UnifiedDir<'a> {
    Ext4(<<Ext4Fs<'a> as FileSystem<'a>>::FileType as File>::DirType<'a>),
    Vfat(<<VFatFS<'a> as FileSystem<'a>>::FileType as File>::DirType<'a>),
    Json(<<JsonFS as FileSystem<'a>>::FileType as File>::DirType<'a>),
    Partition(<<PartitionFS<'a> as FileSystem<'a>>::FileType as File>::DirType<'a>),
}

impl<'a> DirIterator for UnifiedDir<'a> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<DirEntry>, Error> {
        match self {
            UnifiedDir::Ext4(f) => f.next(name),
            UnifiedDir::Json(f) => f.next(name),
            UnifiedDir::Vfat(f) => f.next(name),
            UnifiedDir::Partition(f) => f.next(name),
        }
    }
}

pub enum UnifiedAttr<'a> {
    Ext4(<<Ext4Fs<'a> as FileSystem<'a>>::FileType as File>::AttrType<'a>),
    Vfat(<<VFatFS<'a> as FileSystem<'a>>::FileType as File>::AttrType<'a>),
    Json(<<JsonFS as FileSystem<'a>>::FileType as File>::AttrType<'a>),
    Partition(<<PartitionFS<'a> as FileSystem<'a>>::FileType as File>::AttrType<'a>),
}

impl<'a> IntoIterator for UnifiedAttr<'a> {
    type Item = &'a &'a str;
    type IntoIter = core::slice::Iter<'a, &'a str>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            UnifiedAttr::Ext4(f) => f.into_iter(),
            UnifiedAttr::Json(f) => f.into_iter(),
            UnifiedAttr::Vfat(f) => f.into_iter(),
            UnifiedAttr::Partition(f) => f.into_iter(),
        }
    }
}
impl<'a> Attributes<'a> for UnifiedAttr<'a> {
    fn get(&self, name: &str, buf: &mut [u8]) -> Option<Value> {
        match self {
            UnifiedAttr::Ext4(f) => f.get(name, buf),
            UnifiedAttr::Json(f) => f.get(name, buf),
            UnifiedAttr::Vfat(f) => f.get(name, buf),
            UnifiedAttr::Partition(f) => f.get(name, buf),
        }
    }
}
