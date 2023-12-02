//! Make a FAT filesystem.
#![no_std]

use ap_storage::{check, msg2err, Error, Write, WriteExt};
use ap_storage_vfat::{BiosParameterBlock, ExtBiosParameterBlock16, ExtBiosParameterBlock32, Variant};

/// A VFAT builder.
#[derive(Debug, Clone, Copy)]
pub struct MakeVFatFS {
    align: bool,
    drive: u8,
    label: [u8; 11],
    media: u8,
    num_fats: u8,
    oem: [u8; 8],
    per_cluster: u8,
    reserved: u16,
    root_entries: u16,
    sector_size: u16,
    volume_id: u32,
}

impl Default for MakeVFatFS {
    /// The default config with 4k clusters but only a single FAT.
    fn default() -> Self {
        Self {
            align: true,
            drive: 0x80,
            label: *b"NO NAME    ",
            media: 0xf8,
            num_fats: 1,
            oem: *b" alpico ",
            per_cluster: 8,
            reserved: 1,
            root_entries: 0x200,
            sector_size: 512,
            volume_id: 0,
        }
    }
}

/// Convert a string into a fixed array of bytes.
///
/// The string is padded with spaces and truncated if too long.
fn make_string<const C: usize>(v: &str) -> [u8; C] {
    let n = core::cmp::min(C, v.as_bytes().len());
    let mut res = [b' '; C];
    res[..n].copy_from_slice(&v.as_bytes()[..n]);
    res
}

/// A macro to generate the setters.
macro_rules! setter {
    ($name:ident, $typ:ty, $doc:literal) => {
        #[doc=$doc]
        pub fn $name(&mut self, v: $typ) -> Self {
            self.$name = v;
            *self
        }
    };
}

impl MakeVFatFS {
    setter!(drive, u8, "BIOS drive number.");
    setter!(align, bool, "Align the data area to the cluster.");
    setter!(media, u8, "Media type.");
    setter!(volume_id, u32, "Identification of the filesystem.");
    setter!(root_entries, u16, "Minimum number of root entries for fat12 and fat16.");
    setter!(num_fats, u8, "Number of FAT copies. Zero means one.");
    setter!(reserved, u16, "Number of reserved sectors at the start of the disk.");

    /// The size of the sector in bytes. Must be a power of two and at least 128.
    pub fn sector_size(&mut self, v: u16) -> Result<Self, Error> {
        if !v.is_power_of_two() || v < 128 {
            return Err(msg2err!("sector_size must be a power of two and at least 128"));
        }
        self.sector_size = v;
        Ok(*self)
    }

    /// Sectors per cluster. A power of two larger than 0.
    pub fn per_cluster(&mut self, v: u8) -> Result<Self, Error> {
        if !v.is_power_of_two() || v == 0 {
            return Err(msg2err!("per_clusters must be one of [1,2,4,8,16,32,64,128]"));
        }
        self.per_cluster = v;
        Ok(*self)
    }

    /// Volume label.
    pub fn label(&mut self, v: &str) -> Self {
        self.label = make_string(v);
        *self
    }

    /// OEM field.
    pub fn oem(&mut self, v: &str) -> Self {
        self.oem = make_string(v);
        *self
    }

    /// Return the sector size.
    pub fn get_sector_size(&self) -> u16 {
        self.sector_size
    }
    /// Return the per_cluster variable.
    pub fn get_per_cluster(&self) -> u8 {
        self.per_cluster
    }

    /// Profile for the smallest size with 128 byte sectors.
    pub fn tiny() -> Self {
        Self::default()
            .align(false)
            .num_fats(1)
            .root_entries(1)
            .per_cluster(1)
            .unwrap()
            .sector_size(128)
            .unwrap()
    }

    /// Profile for a small size with 512 byte sectors.
    pub fn small() -> Self {
        Self::default()
            .align(false)
            .num_fats(1)
            .root_entries(16)
            .per_cluster(1)
            .unwrap()
    }

    /// Profile to be most compatible by using  (512GB).
    pub fn compat() -> Self {
        Self::default()
            .reserved(8)
            .num_fats(2)
            .per_cluster(16)
            .unwrap()
            .sector_size(512)
            .unwrap()
    }
    /// Profile for a large disk with 4k sectors and 64k clusters (16 TB).
    pub fn large() -> Self {
        Self::default().per_cluster(16).unwrap().sector_size(4096).unwrap()
    }

    /// Profile for a huge disk with 32k sectors and 4M clusters (128 TB).
    pub fn huge() -> Self {
        Self::default().per_cluster(128).unwrap().sector_size(32768).unwrap()
    }
}

impl MakeVFatFS {
    /// Map the variant to a string.
    fn filesys_type(v: Variant) -> [u8; 8] {
        match v {
            Variant::Fat12 => *b"FAT12   ",
            Variant::Fat16 => *b"FAT16   ",
            Variant::Fat32 => *b"FAT32   ",
        }
    }

    /// Calculate the variant and the fat-size in sectors.
    pub fn calc_variant(&self, sectors: u64) -> Result<(Variant, u64), Error> {
        let sector_size = self.sector_size as u64;
        let reserved_sectors = core::cmp::max(self.reserved, 1) as u64;
        let root_sectors =
            (self.root_entries.next_multiple_of((sector_size / 32) as u16) as u64 * 32).div_ceil(sector_size);
        let per_cluster = self.per_cluster as u64;
        let num_fats = core::cmp::max(self.num_fats, 1) as u64;

        // for the FAT12 and FAT16 variants the root-sectors and the two reserved entries have to be accounted for
        let available_sectors = sectors - core::cmp::min(sectors, reserved_sectors + root_sectors);
        if available_sectors < per_cluster + num_fats {
            return Err(msg2err!("not enough space"));
        }

        // start with FAT12
        let fat_size12 = (available_sectors / per_cluster).div_ceil(sector_size * 2 / 3 + num_fats);
        let cluster12 = (available_sectors - fat_size12 * num_fats) / per_cluster;
        if cluster12 < 4085 {
            return Ok((Variant::Fat12, fat_size12));
        }

        // try FAT16 instead
        let fat_size16 = available_sectors.div_ceil((sector_size / 2 * per_cluster) + num_fats);
        let cluster16 = (available_sectors - fat_size16 * num_fats) / per_cluster;
        if cluster16 < 65525 {
            return Ok((Variant::Fat16, fat_size16));
        }

        // finally FAT32 - this has 28-bits for the cluster number
        let entries_per_cluster = sector_size * per_cluster / 4;
        let fat_size32 = (sectors - reserved_sectors + per_cluster * num_fats).div_ceil(entries_per_cluster + num_fats);
        let cluster32 = (sectors - fat_size32 * num_fats) / per_cluster;
        if cluster32 < 0xfff_fff6 {
            return Ok((Variant::Fat32, fat_size32));
        }
        Err(msg2err!("disk to large"))
    }

    /// Return the sector number where the data area starts without alignment.
    pub fn data_start(&self, variant: Variant, fat_size: u64) -> u64 {
        let mut res = self.reserved as u64 + fat_size * self.num_fats as u64;
        if variant != Variant::Fat32 {
            res += (self.root_entries as u64 * 32).div_ceil(self.sector_size as u64);
        }
        res
    }

    /// Initialize the filesystem.
    pub fn build(&self, disk: &dyn Write, sectors: u32) -> Result<(), Error> {
        let sector_size = self.sector_size as u64;
        let sectors = sectors as u64;
        let (variant, fat_size) = self.calc_variant(sectors)?;

        let reserved_sectors = core::cmp::max(self.reserved, 1);
        let num_fats = core::cmp::max(self.num_fats, 1);
        let root_entries = self.root_entries.next_multiple_of((sector_size / 32) as u16);

        let mut bpb = BiosParameterBlock {
            oem: self.oem,
            bytes_per_sector: self.sector_size,
            sectors_per_cluster: self.per_cluster,
            reserved_sectors,
            num_fats,
            media: self.media,
            ..BiosParameterBlock::default()
        };
        if sectors < 0x10000 {
            bpb.total_sectors16 = sectors as u16;
        } else {
            bpb.total_sectors32 = sectors as u32;
        }
        if variant != Variant::Fat32 {
            bpb.root_entries = root_entries;
            bpb.fat_size16 = fat_size as u16;
        };

        let data_start = self.data_start(variant, fat_size);
        if self.align {
            // align the data-area to the next cluster by reserving more sectors
            bpb.reserved_sectors += (data_start.next_multiple_of(self.per_cluster as u64) - data_start) as u16;
        }

        // clear the Reserved, FAT and Root Directory area
        let root_sectors = (bpb.root_entries as u64 * 32).div_ceil(sector_size);
        let clear_sectors = bpb.reserved_sectors as u64 + fat_size * num_fats as u64 + root_sectors;
        check!(disk.discard_all(0, clear_sectors * sector_size));

        // write the parameter blocks
        disk.write_object(0, bpb)?;
        let ebp16 = ExtBiosParameterBlock16 {
            drive: self.drive,
            bootsig: 0x29,
            volume_id: self.volume_id,
            volume_label: self.label,
            filesys_type: Self::filesys_type(variant),
            ..ExtBiosParameterBlock16::default()
        };
        // the boot magic
        disk.write_object(0x1fe, 0xaa55u16)?;

        if variant != Variant::Fat32 {
            disk.write_object(36, ebp16)?;
        } else {
            let mut ebp32 = ExtBiosParameterBlock32 {
                fat_size32: fat_size as u32,
                ext_flags: 0x80,
                root_cluster: 2,
                backup_boot: if bpb.reserved_sectors > 6 { 6 } else { 0 },
                ext: ebp16,
                ..ExtBiosParameterBlock32::default()
            };
            if sector_size >= 512 && bpb.reserved_sectors > 1 {
                ebp32.fs_info = 1;
            }
            disk.write_object(36, ebp32)?;

            // write the FSINFO structure
            if ebp32.fs_info != 0 {
                let ofs = ebp32.fs_info as u64 * sector_size;
                let cluster = ((sectors - data_start) / self.per_cluster as u64) as u32;
                disk.write_object(ofs, 0x41615252u32)?;
                disk.write_object(ofs + 484, 0x61417272u32)?;
                disk.write_object(ofs + 488, cluster - 1)?;
                disk.write_object(ofs + 492, 3u32)?;
                disk.write_object(ofs + 510, 0xaa55u16)?;
            }

            // write the backup boot sector
            let ofs = ebp32.backup_boot as u64 * sector_size;
            disk.write_object(ofs, bpb)?;
            disk.write_object(ofs + 36, ebp32)?;
            disk.write_object(ofs + 0x1fe, 0xaa55u16)?;
        }

        // write the first two fat entries
        for i in 0..bpb.num_fats {
            let ofs = (bpb.reserved_sectors as u64 + (i as u64) * fat_size) * sector_size;
            match variant {
                Variant::Fat12 => disk.write_object(ofs, 0x800ff8u32)?,
                Variant::Fat16 => disk.write_object(ofs, 0x8000fff8u32)?,
                Variant::Fat32 => {
                    disk.write_object(ofs, 0x0800_0000_ffff_fff8u64)?;
                    disk.write_object(ofs + 8, 0xffff_fff8u64)?;
                }
            }
        }
        Ok(())
    }
}
