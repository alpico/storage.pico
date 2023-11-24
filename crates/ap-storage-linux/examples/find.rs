use ap_storage::{directory::Iterator, file::File, meta::FileType, Error, FileSystem};
use ap_storage_linux::LinuxDisk;
use gumdrop::Options;

#[derive(Debug, Options)]
struct CommandOptions {
    /// Print the help message.
    help: bool,

    /// Show all entries.
    all: bool,

    /// Show the directory-entries as well.
    verbose: bool,

    /// The bytes to skip in the disk file.
    offset: u64,

    /// List the file attributes.
    long: bool,

    /// Maximum depth. Zero means unlimited.
    depth: usize,

    /// Start directory.
    #[options(default = "/")]
    start: String,
}

fn visit(opts: &CommandOptions, f: &impl File, path: &String, depth: usize) -> Result<(), Error> {
    let Some(mut iter) = f.dir() else {
        return Ok(());
    };
    let mut name = [0u8; 256];
    while let Some(entry) = iter.next(&mut name)? {
        if entry.typ == FileType::Unknown || (!opts.all && entry.typ == FileType::Parent) {
            continue;
        }
        let st = core::str::from_utf8(&name[..entry.nlen]).unwrap_or_default();
        if opts.long {
            let f = f.open(entry.offset).unwrap();
            let meta = f.meta();
            print!("{:16x}\t{:16x}\t{}\t", meta.id, meta.size, meta.mtime);
        }
        if opts.verbose {
            print!("{entry:?}\t")
        }
        println!("{path}/{st}");

        if entry.typ == FileType::Directory && depth != 1 {
            let mut child = path.clone();
            child.push('/');
            child.push_str(st);
            let f = f.open(entry.offset).unwrap();
            visit(opts, &f, &child, core::cmp::max(depth, 1) - 1)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let opts = CommandOptions::parse_args_default_or_exit();
    let disk = LinuxDisk::new("/dev/stdin", opts.offset)?;
    let fs = ap_storage_unified::UnifiedFs::new(&disk).ok_or(anyhow::anyhow!("no filesystem found"))?;
    let start = &opts.start;
    let child = fs.root()?.lookup_path(start.as_bytes())?;
    visit(&opts, &child, &"".to_string(), opts.depth)
}
