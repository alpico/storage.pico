//! Disk usage for an ext4 filesystem.

use ap_storage::Error;
use ap_storage_ext4_ro::{Ext4Fs, File};
use ap_storage_linux::memdisk::MemDisk;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to benchmark.
    #[arg(short, long)]
    file: String,

    /// Direct acccess.
    #[arg(short, long, default_value_t = false)]
    no_direct: bool,

    /// Leaf optimization
    #[arg(short, long, default_value_t = false)]
    leaf_optimization: bool,
}

fn visit(dir: &File<'_>) -> Result<(usize, u64), Error> {
    let Some(mut iter) = dir.dir() else {
        return Ok((0, 0));
    };
    let mut buf = [0u8; 255];
    let mut count = 0;
    let mut size = 0;

    while let Ok(entry) = iter.next(&mut buf) {
        let name = &buf[..entry.name_len()];
        if entry.name_len() < 3 && (name == b"" || name == b"." || name == b"..") {
            continue;
        }
        count += 1;
        let child = dir.open(entry.inode())?;
        size += child.size();
        if entry.is_dir() {
            let (x, y) = visit(&child)?;
            count += x;
            size += y;
        }
    }
    Ok((count, size))
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let disk = MemDisk::new(&args.file, !args.no_direct)?;
    let fs = Ext4Fs::mount(&disk, args.leaf_optimization)?;
    let dir = fs.root()?;
    let (count, size) = visit(&dir)?;
    println!("{} {} {}", args.file, count, size);
    Ok(())
}
