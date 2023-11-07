//! Disk usage of the whole filesystem.

use al_mmap::Mmap;
use ap_storage::{Error, Read, directory::Iterator, file::{File, FileType}, FileSystem};
use ap_storage_linux::LinuxDisk;
use ap_storage_memory::ReadSlice;
use gumdrop::Options;

#[derive(Debug, Options)]
struct Args {
    /// Print the help message.
    help: bool,

    /// File to benchmark.
    file: String,

    /// Direct acccess.
    no_direct: bool,

    /// Issue pread requests instead of memory mapping the whole file.
    pread: bool,

    /// Leaf optimization
    leaf_optimization: bool,
}

fn visit(dir: &impl File) -> Result<(usize, u64), Error> {
    let Some(mut iter) = dir.dir() else {
        return Ok((0, 0));
    };

    let mut count = 0;
    let mut size = 0;

    while let Some(entry) = iter.next(&mut [])? {
        if matches!(entry.typ, FileType::Unknown | FileType::Parent) {
            continue;
        }

        count += 1;
        let child = dir.open(entry.offset)?;
        size += child.size();
        if entry.typ == FileType::Directory {
            let (x, y) = visit(&child)?;
            count += x;
            size += y;
        }
    }
    Ok((count, size))
}

fn main() -> Result<(), Error> {
    let args = Args::parse_args_default_or_exit();
    let disk_pread = LinuxDisk::new(&args.file);
    let mmap = Mmap::new(&args.file, !args.no_direct, 0, 0)?;
    let disk_mmap = ReadSlice(mmap.0);
    let disk: &dyn Read = if args.pread { &disk_pread } else { &disk_mmap };

    //let fs1 = ap_storage_ext4_ro::Ext4Fs::new(disk, args.leaf_optimization)?;
    let fs1 = ap_storage_vfat_ro::FatFs::new(disk, 0)?;
    let dir = fs1.root()?;
    let (count, size) = visit(&dir)?;
    println!("{} {} {}", args.file, count, size);
    Ok(())
}
