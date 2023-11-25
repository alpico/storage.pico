use ap_storage::{attr::Attributes, directory::DirIterator, file::File, meta::FileType, Error, FileSystem};
use ap_storage_linux::LinuxDisk;
use gumdrop::Options;

#[derive(Debug, Options)]
struct CommandOptions {
    /// Print the help message.
    help: bool,

    /// Show all entries.
    all: bool,

    /// Show the raw directory-entries.
    raw: bool,

    /// The bytes to skip in the disk file.
    offset: u64,

    /// List the file metadata.
    meta: bool,

    /// List the file attributes.
    attr: bool,

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
        println!("{path}/{st}");
        if opts.meta || opts.attr {
            let f = f.open(entry.offset).unwrap();
            if opts.meta {
                let meta = f.meta();
                println!("\tid\t{:x}", meta.id);
                println!("\tsize\t{}", meta.size);
                println!("\tmtime\t{:x}", meta.mtime);
                println!("\ttype\t{:?}", entry.typ);
            }
            if opts.attr {
                let mut attr = f.attr();
                let mut name = [0u8; 32];
                let mut value = [0u8; 256];
                while let Some(entry) = attr.next(&mut name, &mut value)? {
                    let k =
                        core::str::from_utf8(&name[..core::cmp::min(name.len(), entry.name_len)]).unwrap_or_default();
                    let v = core::str::from_utf8(&value[..core::cmp::min(value.len(), entry.value_len)])
                        .unwrap_or_default();
                    println!("\t{k}\t{v}")
                }
            }
        }
        if opts.raw {
            println!("\tentry\t{entry:?}\t")
        }

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
