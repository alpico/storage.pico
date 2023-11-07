use ap_storage::{Error, FileType};
use ap_storage_linux::LinuxDisk;
use ap_storage_vfat_ro::{FatFs, File};

fn visit(f: &File, path: String) -> Result<(), Error> {
    let mut iter = f.dir(false).unwrap();
    let mut name = [0u8; 256];
    while let Some(entry) = iter.next(&mut name)? {
        if matches!(entry.typ, FileType::Unknown | FileType::Parent) {
            continue;
        }
        let st = core::str::from_utf8(&name[..entry.nlen]).unwrap_or_default();
        println!("{path}/{st}\t{entry:?}");

        if entry.typ == FileType::Directory {
            let mut child = path.clone();
            child.push_str("/");
            child.push_str(st);
            visit(&f.open(entry.offset).unwrap(), child)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let disk = LinuxDisk::new("/dev/stdin");
    let fs = FatFs::new(&disk, 0)?;
    dbg!(&fs);
    let root = fs.root();
    dbg!(&root);

    visit(&root, "".to_string())?;
    // XXX lookup file
    Ok(())
}
