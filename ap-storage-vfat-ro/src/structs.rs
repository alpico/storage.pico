//! On-disk structs.

#[derive(Clone, Copy, Default, PartialEq)]
#[repr(packed)]
pub struct DirectoryEntry {
    pub name: [u8; 11],
    pub attr: u8,
    pub _x: [u8; 8],
    pub cluster_hi: u16,
    pub wtime: u32,
    pub cluster_lo: u16,
    pub size: u32,
}

impl DirectoryEntry {
    pub fn is_dir(&self) -> bool {
        self.attr & 0x10 != 0
    }

    pub fn cluster(&self) -> u32 {
        // Volume ID?
        if self.attr & 0x8 != 0 {
            return 0;
        }
        (self.cluster_hi as u32) << 16 | self.cluster_lo as u32
    }

    pub fn size(&self) -> u32 {
        let mut res = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.size)) };
        if self.is_dir() && res == 0 {
            // The size of a directory is typically zero.
            // But we know there cannot be more then 64k entries per directory.
            res = 65536 * 32
        }
        res
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
}

impl core::fmt::Debug for DirectoryEntry {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
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

/// The minimal BIOS Parameter Block as present in the first sector of the disk.
#[derive(Clone, Copy, Debug)]
#[repr(packed)]
pub struct BiosParameterBlock {
    _x: [u8; 11],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub root_entries: u16,
    pub total_sectors16: u16,
    pub media: u8,
    pub fat_size16: u16,
    _y: [u8; 8],
    pub total_sectors32: u32,
    pub fat_size32: u32,
    pub ext_flags: u16,
    _z: u16,
    pub root_cluster: u32,
}
