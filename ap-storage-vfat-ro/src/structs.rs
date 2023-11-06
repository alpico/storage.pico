//! On-disk structs.

#[derive(Clone, Copy, Debug, Default)]
#[repr(packed)]
pub struct DirEntry  {
    pub name: [u8; 11],
    pub attr: u8,
    pub _x: [u8; 8],
    pub cluster_hi: u16,
    pub wtime: u32,
    pub cluster_lo: u16,
    pub size: u32,
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
