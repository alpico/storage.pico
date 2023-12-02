//! Search for files in a file-system recursively.

use ap_storage::{
    attr::{Attributes, Value},
    directory::DirIterator,
    file::File,
    file::FileType,
    msg2err, Error, FileSystem,
};
use ap_storage_linux::LinuxDiskRO;
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

    /// List the file attributes.
    attr: bool,

    /// Maximum depth. Zero means unlimited.
    depth: usize,

    /// Start directories.
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
        if opts.attr {
            let f = f.open(entry.offset).unwrap();
            let attr = f.attr();
            let mut value = [0u8; 256];
            for name in f.attr() {
                match attr.get(name, &mut value).unwrap() {
                    Value::U64(v) => println!("\t{name}\t{:#x}", v),
                    Value::I64(v) => println!("\t{name}\t{}", v),
                    Value::Bool(v) => println!("\t{name}\t{:?}", v),
                    Value::Raw(count) => {
                        let v = core::str::from_utf8(&value[..core::cmp::min(value.len(), count)]).unwrap_or_default();
                        println!("\t{name}\t{v}");
                    }
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
    let disk = LinuxDiskRO::new("/dev/stdin", opts.offset)?;
    let fs = ap_storage_unified::UnifiedFs::new(&disk).ok_or(msg2err!("no filesystem found"))?;
    let start = &opts.start;
    let child = fs.root()?.lookup_path(start.as_bytes())?;
    visit(&opts, &child, &"".to_string(), opts.depth)
}
