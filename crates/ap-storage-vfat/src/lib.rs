//! On-disk structures for FAT filesystems.
//!
//! ## Features
//! - `fat-plus` - support upto 256 GB large files

#![no_std]
#![feature(byte_slice_trim_ascii)]

// This crate contains on-disk structures that are already defined in various specifications.
// There is no need to copy-paste their docs here.
#![allow(missing_docs)]

mod long_entry;
use ap_util_date::{dos_date2ts, dos_time2ts, Time};
pub use long_entry::LongEntry;

/// The different FAT variants.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Variant {
    Fat12 = 12,
    Fat16 = 16,
    Fat32 = 32,
}

/// Directory entry.
#[derive(Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct DirectoryEntry {
    pub name: [u8; 11],
    pub attr: u8,
    pub res: u8,
    /// birth time in 10 millisecond increments
    pub btenthms: u8,
    pub btime: u16,
    pub bdate: u16,
    pub adate: u16,
    pub cluster_hi: u16,
    pub mtime: u16,
    pub mdate: u16,
    pub cluster_lo: u16,
    pub size: u32,
}

impl DirectoryEntry {
    /// Is this entry a directory?
    pub fn is_dir(&self) -> bool {
        self.attr & 0x10 != 0
    }

    /// Get the attributes field.
    pub fn attr(&self) -> u8 {
        self.attr
    }

    /// Return the cluster number.
    pub fn cluster(&self) -> u32 {
        // Volume ID?
        if self.attr & 0x8 != 0 {
            return 0;
        }
        (self.cluster_hi as u32) << 16 | self.cluster_lo as u32
    }

    /// Calculate the size of the file.
    pub fn size(&self) -> u64 {
        let mut res = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.size)) };
        if self.is_dir() && res == 0 {
            // The size of a directory is typically zero.
            // But there will never be more then 64k entries per directory.
            res = 65536 * 32
        }
        if cfg!(features = "fat-plus") {
            let extra = (self.res & 0x7) | (self.res & 0xd >> 2);
            return res as u64 | (extra as u64) << 32;
        }
        res as u64
    }

    /// Returns the short-name of the directory.
    pub fn name(&self) -> [u8; 12] {
        let mut res = [b' '; 12];

        // unused entries?
        if matches!(self.name[0], 0 | 0xe5) {
            return res;
        }

        // the parts are padded independently
        let name = self.name[..8].trim_ascii_end();
        let ext = self.name[8..].trim_ascii_end();
        res[..name.len()].copy_from_slice(name);
        if !ext.is_empty() {
            res[name.len()] = b'.';
            res[name.len() + 1..name.len() + 1 + ext.len()].copy_from_slice(ext);
        }
        // magic value to support KANJI encoding
        if res[0] == 0x05 {
            res[0] = 0xe5;
        }
        res
    }

    /// Calculate the name checksum for long-entries.
    pub fn checksum(&self) -> u8 {
        let mut res = self.name[0] as usize;
        for i in 1..11 {
            res = ((res & 0x1) << 7 | (res & 0xff) >> 1) + self.name[i] as usize;
        }
        (res & 0xff) as u8
    }

    /// Return the mtime in nanoseconds since 1970.
    pub fn mtime(&self) -> Time {
        (dos_date2ts(self.mdate) + dos_time2ts(self.mtime)) * 1_000_000_000
    }

    /// Return the birth time in nanoseconds since 1970.
    pub fn btime(&self) -> Time {
        (dos_date2ts(self.bdate) + dos_time2ts(self.btime)) * 1_000_000_000 + self.btenthms as Time * 10_000_000
    }

    /// Return the atime in nanoseconds since 1970.
    pub fn atime(&self) -> Time {
        dos_date2ts(self.mdate) * 1_000_000_000
    }
}
impl core::fmt::Debug for DirectoryEntry {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            fmt,
            "DirectoryEntry({:x}, {:x}+{:x}, '{}')",
            self.attr,
            self.cluster(),
            self.size(),
            core::str::from_utf8(&self.name()).unwrap()
        )
    }
}

/// The BIOS Parameter Block as present in the first sector of the disk.
#[derive(Clone, Copy, Debug, Default)]
#[repr(packed)]
pub struct BiosParameterBlock {
    pub jmp: [u8; 3],
    pub oem: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub root_entries: u16,
    pub total_sectors16: u16,
    pub media: u8,
    pub fat_size16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors32: u32,
}

/// The extension as present on fat12 and fat16 volumes.
#[derive(Clone, Copy, Debug, Default)]
#[repr(packed)]
pub struct ExtBiosParameterBlock16 {
    pub drive: u8,
    pub res: u8,
    pub bootsig: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub filesys_type: [u8; 8],
}

/// The extension as present on fat32 volumes.
#[derive(Clone, Copy, Debug, Default)]
#[repr(packed)]
pub struct ExtBiosParameterBlock32 {
    pub fat_size32: u32,
    pub ext_flags: u16,
    pub version: u16,
    pub root_cluster: u32,
    pub fs_info: u16,
    pub backup_boot: u16,
    pub res: [u8; 12],
    pub ext: ExtBiosParameterBlock16,
}
