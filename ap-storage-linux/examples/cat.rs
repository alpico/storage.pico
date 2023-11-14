//! Output a file.

use ap_storage::{file::File, Error, FileSystem, Read};
use ap_storage_linux::LinuxDisk;
use gumdrop::Options;
use std::io::Write;

#[derive(Debug, Options)]
struct CommandOptions {
    /// Print the help message.
    help: bool,

    /// Buffer size.
    #[options(default = "65536")]
    buffer: usize,

    /// The bytes to skip.
    skip: u64,

    /// The bytes to skip in the disk file.
    offset: u64,

    /// Maximum number of bytes to read.
    size: MaxSize,

    /// Name of the file.
    #[options(default = "/")]
    start: String,
}

/// An u64 that is default u64::MAX.
#[derive(Debug)]
struct MaxSize(pub u64);
impl Default for MaxSize {
    fn default() -> Self {
        Self(u64::MAX)
    }
}

impl core::str::FromStr for MaxSize {
    type Err = core::num::ParseIntError;
    fn from_str(v: &str) -> Result<Self, <Self as core::str::FromStr>::Err> {
        Ok(Self(str::parse::<u64>(v)?))
    }
}

fn main() -> Result<(), Error> {
    let opts = CommandOptions::parse_args_default_or_exit();
    let disk = LinuxDisk::new("/dev/stdin", opts.offset);
    let fs =
        ap_storage_unified::UnifiedFs::new(&disk).ok_or(anyhow::anyhow!("no filesystem found"))?;
    let start = &opts.start;
    let file = fs.root()?.lookup_path(start.as_bytes())?;

    let mut buf = vec![0; opts.buffer];
    let mut offset = opts.skip;
    let mut stdout = std::io::stdout();

    let mut size = opts.size.0;
    while size != 0 {
        let maxn = core::cmp::min(buf.len() as u64, size) as usize;
        match file.read_bytes(offset, &mut buf[..maxn])? {
            0 => break,
            n => {
                stdout.write_all(&buf[..n])?;
                offset += n as u64;
                size -= n as u64;
            }
        }
    }
    Ok(())
}
