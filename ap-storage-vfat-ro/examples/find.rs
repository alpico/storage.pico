use ap_storage::{directory::Iterator, file::File, file::FileType, Error, FileSystem};
use ap_storage_linux::LinuxDisk;
use ap_storage_vfat_ro::FatFs;
use gumdrop::Options;

#[derive(Debug, Options)]
struct CommandOptions {
    /// Print the help message.
    help: bool,
    /// Show all entries.
    all: bool,
}

fn visit(opts: &CommandOptions, f: &impl File, path: String) -> Result<(), Error> {
    let mut iter = f.dir().unwrap();
    let mut name = [0u8; 256];
    while let Some(entry) = iter.next(&mut name)? {
        if entry.typ == FileType::Unknown || !opts.all && entry.typ == FileType::Parent {
            continue;
        }
        let st = core::str::from_utf8(&name[..entry.nlen]).unwrap_or_default();
        println!("{path}/{st}");

        if entry.typ == FileType::Directory {
            let mut child = path.clone();
            child.push('/');
            child.push_str(st);
            let f = f.open(entry.offset).unwrap();
            visit(opts, &f, child)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let opts = CommandOptions::parse_args_default_or_exit();
    let disk = LinuxDisk::new("/dev/stdin");
    let fs = FatFs::new(&disk, 0)?;
    let root = fs.root()?;
    visit(&opts, &root, "".to_string())?;
    Ok(())
}
