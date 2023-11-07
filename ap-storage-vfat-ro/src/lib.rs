//! Read from fat{12,16,32} filesystem.
#![feature(byte_slice_trim_ascii)]

mod structs;
use structs::{DirEntry, BiosParameterBlock};
use ap_storage::{Read, ReadExt, Error};
mod file;
pub use file::File;

#[derive(Clone)]
pub struct FatFs<'a> {
    disk: &'a dyn Read,
    /// Sector size in bytes.
    sector_size: u32,
    /// Sectors per cluster.
    sectors_per_cluster: u32,
    /// The filesystem type: 12,16 or 32.
    fat_type: u32,
    /// The sector where the FAT starts.
    fat_start: u32,
    /// The mask for the FAT entries.
    fat_mask: u32,
    /// The start of the root directory.
    root_start: u32,
    /// Virtual directory entry for the root directory.
    root_dir: DirEntry,
    /// The uuid field.
    uuid: u32,
}


impl core::fmt::Debug for FatFs<'_> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "FatFs{}(uuid {:x?}, cluster {} x {})", self.fat_type,self.uuid, self.sectors_per_cluster, self.sector_size)
    }
}

impl<'a> FatFs<'a> {
    /// Mount the filesystem.
    pub fn new(disk: &'a dyn Read, sb_offset: u64) -> Result<Self, Error> {
        let buf: [u8; 512] = disk.read_object(sb_offset)?;
        let bpb = unsafe { *(buf.as_ptr() as *const BiosParameterBlock) };

        
        // validate the super-block
        if buf[511] != 0xaa || buf[510] != 0x55 {
            return Err(anyhow::anyhow!("boot signature"));
        }
        if bpb.bytes_per_sector > 4096 || bpb.bytes_per_sector < 512 || (bpb.bytes_per_sector & bpb.bytes_per_sector - 1) != 0 {
            return Err(anyhow::anyhow!("bytes per sector"));
        }
        if bpb.sectors_per_cluster == 0 || (bpb.sectors_per_cluster & bpb.sectors_per_cluster - 1) != 0 {
            return Err(anyhow::anyhow!("bytes per cluster"));
        }
        if bpb.reserved_sectors == 0 {
            return Err(anyhow::anyhow!("reserved sectors"));
        }
        if bpb.num_fats == 0 {
            return Err(anyhow::anyhow!("FAT count"));
        }
        if bpb.media < 0xf8 && bpb.media != 0xf0 {
            return Err(anyhow::anyhow!("media byte"));
        }

        // select the 16-bit or 32-bit version
        let left_or = |x, y| if x == 0 { y } else { x as u32 };
        
        // calculate the constants
        let sector_size = bpb.bytes_per_sector as u32;
        let root_sectors = (((bpb.root_entries as u32) << 5) + (bpb.bytes_per_sector as u32 - 1)) / sector_size;
        let sectors_per_cluster = bpb.sectors_per_cluster as u32;
        let mut fat_start = bpb.reserved_sectors as u32;
        let root_start = fat_start + (bpb.num_fats as u32)*(left_or(bpb.fat_size16, bpb.fat_size32));
        let clusters = (left_or(bpb.total_sectors16, bpb.total_sectors32) - (root_start + root_sectors)) / sectors_per_cluster;
        let fat_type = if clusters < 4085 { 12} else if clusters < 65525 { 16 } else { 32 };
        let fat_mask = 0x0fffffff & (!0u32 >> (32 - fat_type));

        let uuid: u32 = unsafe { core::ptr::read_unaligned(buf.as_ptr().add(if fat_type == 32 { 67 } else { 39 }) as *const u32) };

        // init root directory entry
        let root_dir = DirEntry {
            attr: 0x10,
            name: *b"..         ",
            size: root_sectors * sector_size,
            cluster_lo: if fat_type == 32 { (bpb.root_cluster & 0xffff) as u16 } else { 1 },
            cluster_hi: if fat_type == 32 { (bpb.root_cluster >> 16) as u16 } else { 0 },
            ..Default::default()
        };
            
        // check for active fat
        if fat_type == 32 && (bpb.ext_flags & 0x80) != 0 && bpb.ext_flags & 0xf < bpb.num_fats as u16 {
            fat_start = fat_start + ((bpb.ext_flags & 0xf) as u32) * bpb.fat_size32
        }

        Ok(Self { disk, sector_size, sectors_per_cluster, fat_type, fat_start, fat_mask, root_start, root_dir, uuid })
    }

    pub fn root(&self) -> File {
        File::new(self.root_dir)
    }
}
