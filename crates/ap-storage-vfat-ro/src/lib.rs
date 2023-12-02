//! Read from FAT filesystem.
//!
//! - support for FAT 12,16,32
//! - `long-name` feature for larger filename
//! - wide-range of sectors - 128 to 32k
//! - huge clusters - upto 4M
//! - files upto 256GB with the `fat-plus` feature.

#![no_std]
#![feature(byte_slice_trim_ascii)]

use ap_storage::{Error, FileSystem, Offset, Read, ReadExt, msg2err};
use ap_storage_vfat::*;

mod attr;
mod dir;
mod file;

/// The mount options.
#[derive(Default, Clone)]
pub struct Options {
    /// The offset for the superblock may point to a backup.
    pub sb_offset: Offset,
    /// Ignore long directory entries.
    pub ignore_long_name: bool,
    /// Lowercase the short names.
    pub lower_short_name: bool,
}

/// An VFAT filesystem.
#[derive(Clone)]
pub struct VFatFS<'a> {
    disk: &'a dyn Read,
    /// bytes per cluster.
    cluster_size: u32,
    /// The number of clusters in the data-area.
    clusters: u32,
    /// The filesystem variant.
    variant: Variant,
    /// The offset where the FAT starts.
    fat_start: Offset,
    /// The mask for the FAT entries.
    fat_mask: u32,
    /// The start of the data area -> cluster 2.
    data_start: Offset,
    /// The offset where the root-region starts.
    root_start: Offset,
    /// size of the root-directory for u32
    root_size: u32,
    /// the root cluster for fat32
    root_cluster: u32,
    /// The uuid field.
    uuid: u32,
    /// Mount options,
    options: Options,
}

impl core::fmt::Debug for VFatFS<'_> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            fmt,
            "VFatFS{}( uuid {:x?}, bs {})",
            self.variant as usize, self.uuid, self.cluster_size
        )
    }
}

impl<'a> VFatFS<'a> {
    /// Mount the filesystem.
    pub fn new(disk: &'a dyn Read, options: Options) -> Result<Self, Error> {
        let buf: [u8; 512] = disk.read_object(options.sb_offset)?;
        let bpb = unsafe { *(buf.as_ptr() as *const BiosParameterBlock) };
        let ebp16 = unsafe { *(buf.as_ptr().add(36) as *const ExtBiosParameterBlock16) };
        let ebp32 = unsafe { *(buf.as_ptr().add(36) as *const ExtBiosParameterBlock32) };

        // validate the super-block
        if buf[511] != 0xaa || buf[510] != 0x55 {
            return Err(msg2err!("boot signature"));
        }
        if bpb.bytes_per_sector < 128 || !bpb.bytes_per_sector.is_power_of_two() {
            return Err(msg2err!("bytes per sector"));
        }
        if bpb.sectors_per_cluster == 0 || !bpb.sectors_per_cluster.is_power_of_two() {
            return Err(msg2err!("bytes per cluster"));
        }
        if bpb.reserved_sectors == 0 {
            return Err(msg2err!("reserved sectors"));
        }
        if bpb.num_fats == 0 {
            return Err(msg2err!("FAT count"));
        }
        if bpb.media < 0xf8 && bpb.media != 0xf0 {
            return Err(msg2err!("media byte"));
        }

        // select the 16-bit or 32-bit version of a field
        let left_or = |x, y| if x == 0 { y } else { x as u32 };

        // calculate the constants
        let sector_size = bpb.bytes_per_sector as u32;
        let root_sectors = (((bpb.root_entries as u32) << 5) + (sector_size - 1)) / sector_size;
        let sectors_per_cluster = bpb.sectors_per_cluster as u32;
        let mut fat_start_sector = bpb.reserved_sectors as u32;
        let root_start = fat_start_sector + (bpb.num_fats as u32) * (left_or(bpb.fat_size16, ebp32.fat_size32));
        let data_start = (root_start + root_sectors) as Offset * sector_size as Offset;
        let clusters =
            (left_or(bpb.total_sectors16, bpb.total_sectors32) - (root_start + root_sectors)) / sectors_per_cluster;
        let variant = match clusters {
            x if x < 4085 => Variant::Fat12,
            x if x < 65525 => Variant::Fat16,
            _ => Variant::Fat32,
        };
        let fat_mask = 0x0fffffff & (!0u32 >> (32 - variant as u32));
        let uuid = match variant {
            Variant::Fat32 => ebp32.ext.volume_id,
            _ => ebp16.volume_id,
        };

        // check for active fat
        if variant == Variant::Fat32 && (ebp32.ext_flags & 0x80) != 0 && ebp32.ext_flags & 0xf < bpb.num_fats as u16 {
            fat_start_sector += ((ebp32.ext_flags & 0xf) as u32) * ebp32.fat_size32
        }
        let root_cluster = match variant {
            Variant::Fat32 => ebp32.root_cluster,
            _ => 0,
        };

        Ok(Self {
            disk,
            cluster_size: sector_size * sectors_per_cluster,
            clusters,
            data_start,
            variant,
            fat_start: fat_start_sector as Offset * sector_size as Offset,
            fat_mask,
            root_start: root_start as Offset * sector_size as Offset,
            root_size: root_sectors * sector_size,
            root_cluster,
            uuid,
            options,
        })
    }

    /// Follow the fat one entry at a time.
    fn follow_fat(&self, cluster: u32) -> Result<u32, Error> {
        if cluster == 0 || cluster > self.clusters + 2 {
            return Err(msg2err!("eof"));
        }
        let ofs = self.fat_start + cluster as Offset * self.variant as Offset / 8;

        let mut value = match self.variant {
            Variant::Fat32 => self.disk.read_object::<u32>(ofs)?,
            _ => self.disk.read_object::<u16>(ofs)? as u32,
        };

        // this is the odd-case
        if self.variant == Variant::Fat12 && cluster & 1 != 0 {
            value >>= 4;
        }
        Ok(value & self.fat_mask)
    }
}

impl<'a> FileSystem<'a> for VFatFS<'a> {
    type FileType = file::File<'a>;
    fn root(&'a self) -> Result<Self::FileType, Error> {
        let root_dir = DirectoryEntry {
            attr: 0x10,
            name: *b"..         ",
            size: self.root_size,
            ..Default::default()
        };

        Ok(file::File::new(self, root_dir, self.root_start))
    }
}
