use ap_storage::Error;
use ap_storage_linux::LinuxDisk;
use ap_storage_vfat_ro::FatFs;

fn main() -> Result<(), Error> {
    let disk = LinuxDisk::new("/dev/stdin");
    let fs = FatFs::new(&disk, 0)?;
    dbg!(&fs);
    let root = fs.root();
    dbg!(&root);

    // XXX lookup file
    let mut iter = root.dir().unwrap();
    let mut name = [0u8; 255];
    while let Ok(entry) = iter.next(&mut name) {
        println!("{}\t{entry:?}", core::str::from_utf8(&name[..entry.nlen]).unwrap_or_default())
    }
    Ok(())
}
