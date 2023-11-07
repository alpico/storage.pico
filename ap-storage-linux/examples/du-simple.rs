//! Disk usage for an ext4 filesystem.

use al_mmap::Mmap;
use ap_storage::{Error, Read, directory::Iterator, file::FileType};
use ap_storage_ext4_ro::{Ext4Fs, File};
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

fn visit(dir: &File<'_>, fs: &Ext4Fs) -> Result<(usize, u64), Error> {
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
        let child = File::new(fs, entry.id)?;
        size += child.size();
        if entry.typ == FileType::Directory {
            let (x, y) = visit(&child, fs)?;
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

    let fs = Ext4Fs::new(disk, args.leaf_optimization)?;
    let dir = fs.root()?;
    let (count, size) = visit(&dir, &fs)?;
    println!("{} {} {}", args.file, count, size);
    Ok(())
}
