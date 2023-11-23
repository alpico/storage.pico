//! Build a vfat filesystem.
//!
//!
//! # Options
//!
//! The FAT definition is quite flexible to support a wide range of
//! devices. The most imporant options are:
//!
//! - `sector-size`  - a power-of two - usually 512 bytes but up to 4096 is standardized.
//! - `per-cluster`  - a power-of two between 1 and 128.  Defines the cluster-size.
//! - `root-entries` - usually 512 - used for FAT16 and FAT12 to define the size of the root-directory
//! - `reserved`     - the number of reserved sectors at the beginning of the disk.
//!
//! # Assumptions
//! -
use ap_storage::{Error, Offset, Read, ReadExt};
use ap_storage_linux::LinuxDiskRW;
use ap_storage_vfat_mkfs::MakeVFatFS;
use gumdrop::Options;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Options)]
struct CommandOptions {
    /// Print the help message.
    help: bool,

    /// The bytes to skip in the disk file.
    offset: Offset,

    /// BIOS drive number
    #[options(default = "128")]
    drive: u8,

    /// Do not align the data area to the cluster.
    no_align: bool,

    /// Media type
    #[options(default = "248")]
    media: u8,

    /// Volume label.
    #[options(default = "NO NAME")]
    label: String,

    /// Number of fat copies.
    num_fats: u8,

    /// OEM field.
    #[options(default = " alpico")]
    oem: String,

    /// One of [1,2,4,8,16,32,64,128].
    #[options(default = "8")]
    per_cluster: u8,

    /// Minimum of reserved sectors.
    reserved: u16,

    /// Number of root entries for fat12 and fat16 variants.
    #[options(default = "512")]
    root_entries: u16,

    /// Typically one of [512,1024,2048,4096] bytes.
    #[options(default = "512")]
    sector_size: u16,

    /// Volume id.
    volume_id: u32,
}

/// Get a randomized volume ID.
fn rand_volume_id() -> u32 {
    let nsec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("no system time")
        .subsec_nanos();
    (nsec % 0xffff_ffff) + 1
}

fn main() -> Result<(), Error> {
    let opts = CommandOptions::parse_args_default_or_exit();
    let builder = MakeVFatFS::default()
        .align(!opts.no_align)
        .drive(opts.drive)
        .label(&opts.label)
        .media(!opts.media)
        .num_fats(opts.num_fats)
        .oem(&opts.oem)
        .per_cluster(opts.per_cluster)?
        .reserved(opts.reserved)
        .root_entries(opts.root_entries)
        .sector_size(opts.sector_size)?
        .volume_id(if opts.volume_id == 0 {
            rand_volume_id()
        } else {
            opts.volume_id
        });

    let disk = LinuxDiskRW::new("/dev/stdin", opts.offset)?;
    let r = &disk as &dyn Read;
    builder.build(&disk, r.detect_size())
}
