//! Disk usage of the whole filesystem.

use al_mmap::Mmap;
use ap_storage::{
    directory::Iterator,
    file::File,
    meta::FileType,
    Error, FileSystem, Read,
};
use ap_storage_linux::LinuxDisk;
use ap_storage_memory::ReadSlice;
use gumdrop::Options;

#[derive(Debug, Options)]
struct CommandOptions {
    /// Print the help message.
    help: bool,

    /// Direct acccess.
    no_direct: bool,

    /// Issue pread requests instead of memory mapping the whole file.
    pread: bool,

    /// Leaf optimization
    leaf_optimization: bool,

    /// Start directory.
    #[options(default = "/")]
    start: String,
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
        size += child.meta().size;
        if entry.typ == FileType::Directory {
            let (x, y) = visit(&child)?;
            count += x;
            size += y;
        }
    }
    Ok((count, size))
}

fn main() -> Result<(), Error> {
    let opts = CommandOptions::parse_args_default_or_exit();
    let disk_pread = LinuxDisk::new("/dev/stdin", 0);
    let mmap = Mmap::new("/dev/stdin", !opts.no_direct, 0, 0)?;
    let disk_mmap = ReadSlice(mmap.0);
    let disk: &dyn Read = if opts.pread { &disk_pread } else { &disk_mmap };

    let fs =
        ap_storage_unified::UnifiedFs::new(disk).ok_or(anyhow::anyhow!("no filesystem found"))?;
    let dir = fs.root()?.lookup_path(opts.start.as_bytes())?;
    let (count, size) = visit(&dir)?;
    println!("{}\t{}\t{}", opts.start, count, size);
    Ok(())
}
