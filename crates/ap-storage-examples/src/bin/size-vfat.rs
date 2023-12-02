//! Estimate the size a VFAT filesystem.
//!
//! It is hard to efficiently resize a VFAT filesystem.  In many cases
//! this requires moving all clusters and fixing all directory entries
//! to resize the FAT as well.
//!
//! However it is relatively simple to calculate the required size for
//! such an filesystem given the directory layout and the file sizes.

use ap_storage::{msg2err, Error, FileSystem, file::{File, FileType}, attr::{Attributes, SIZE}, directory::DirIterator};
use ap_storage_vfat::Variant;
use ap_storage_linux::LinuxDiskRO;
use gumdrop::Options;

#[derive(Debug, Options)]
struct CommandOptions {
    /// Print the help message.
    help: bool,

    /// Verbose output.
    verbose: bool,

    /// The bytes to skip in the disk file.
    offset: u64,

    /// Sectors per cluster. Power of two in the range {1,128}.
    #[options(meta = "N", default = "8")]
    per_cluster: u8,

    /// Power of two in the range {128,32768}. Typically 512 or 4096.
    #[options(meta = "N", default = "512")]
    sector_size: u16,

    /// Number of FAT copies.
    #[options(meta = "N", default = "1")]
    num_fats: u8,

    /// Start directory.
    #[options(default = "/")]
    start: String,

    /// Additional sectors to reserve.
    #[options(meta = "N")]
    add: u64,
}

/// Count the clusters and directory entries per file.
fn count(f: &impl File, cluster_size: u64) -> Result<(u64, u64), Error> {
    let Some(mut iter) = f.dir() else {
        let bytes = f.attr().get(SIZE, &mut []).ok_or(msg2err!("no size"))?.as_u64().unwrap_or_default();
        return Ok((bytes.div_ceil(cluster_size), 0));
    };
    let mut num: u64 = 0;
    let mut res = 0u64;

    let mut name = [0u8; 256];
    while let Some(entry) = iter.next(&mut name)? {
        match entry.typ {
            FileType::Unknown => {}
            FileType::Parent => { num += 1 },
            _ => {
                // how many long-entries do we need?
                let sname = core::str::from_utf8(&name[..core::cmp::min(name.len(), entry.nlen)]);
                let nlen = sname.unwrap_or_default().encode_utf16().count();

                // we don't check for valid 8.3 case where no long-entry is needed
                num += 1 + nlen.div_ceil(13) as u64;

                // measure recursively
                let f = f.open(entry.offset).unwrap();
                let (clusters, entries) = count(&f, cluster_size)?;
                res += clusters + (entries*32).div_ceil(cluster_size);
            }
        }
    }
    
    Ok((res, num))
}


fn main() -> Result<(), Error> {
    let opts = CommandOptions::parse_args_default_or_exit();
    let disk = LinuxDiskRO::new("/dev/stdin", opts.offset)?;
    let fs = ap_storage_unified::UnifiedFs::new(&disk).ok_or(msg2err!("no filesystem found"))?;

    let start = &opts.start;
    let child = fs.root()?.lookup_path(start.as_bytes())?;
    let cluster_size = opts.sector_size as u64 * opts.per_cluster as u64;
    
    let (mut clusters, entries) = count(&child, cluster_size)?;
    let mut root_sectors = (entries * 32).div_ceil(opts.sector_size as u64);
    let root_clusters = root_sectors.div_ceil(opts.per_cluster as u64);
    if clusters + root_clusters >= 65525 {
        clusters += root_clusters;
        root_sectors = 0;
    }
    let variant = match clusters {
        x if x < 4085 => Variant::Fat12,
        x if x < 65525 => Variant::Fat16,
        x if x < 0xfff_fff6 => Variant::Fat32,
        _ => { return Err(msg2err!("disk to large - should increase the cluster size")) }
    };
    let fat_size = (clusters * variant as u64 / 8).div_ceil(opts.sector_size as u64);
    let clusters = clusters + (1 + fat_size * opts.num_fats as u64 + root_sectors).div_ceil(opts.per_cluster as u64);
    let sectors = opts.add + clusters * opts.per_cluster as u64;
    if opts.verbose {
        dbg!(sectors, fat_size, clusters, root_sectors);
    }
    println!("{}\t{}", sectors * opts.sector_size as u64, entries);
    Ok(())
}
