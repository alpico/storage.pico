//! Build a vfat filesystem.
//!
//!
//! # Options
//!
//! The FAT definition is quite flexible to support a wide range of
//! devices. The most imporant options are:
//!
//! - `sector-size`  - a power-of two - usually 512 bytes but up to 4096 is standardized.
//! - `per-cluster`  - a power-of two between 1 and 128.  Defines the cluster-size.
//! - `root-entries` - usually 512 - used for FAT16 and FAT12 to define the size of the root-directory
//! - `reserved`     - the number of reserved sectors at the beginning of the disk.
//!
//! # Assumptions
//! -
use ap_storage::{Error, Offset, Read, ReadExt};
use ap_storage_linux::LinuxDiskRW;
use ap_storage_vfat_mkfs::MakeVFatFS;
use core::str::FromStr;
use gumdrop::Options;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Options)]
struct CommandOptions {
    /// Print the help message.
    help: bool,
    /// Verbose output.
    verbose: bool,
    /// Do not write.
    dry_run: bool,
    /// The bytes to skip in the disk file.
    #[options(meta = "N")]
    offset: Offset,

    /// BIOS drive number.
    #[options(meta = "N")]
    drive: UnsetField<u8>,
    /// Align the data area to the cluster.
    align: UnsetField<bool>,
    /// Media type
    #[options(meta = "N")]
    media: UnsetField<u8>,
    /// Volume label.
    label: UnsetField<String>,
    /// Number of fat copies. Typically one or two.
    #[options(meta = "N")]
    num_fats: UnsetField<u8>,
    /// OEM field.
    oem: UnsetField<String>,
    /// Sectors per cluster. One of {1,2,4,8,16,32,64,128}.
    #[options(meta = "N")]
    per_cluster: UnsetField<u8>,
    /// Minimum of reserved sectors.
    #[options(meta = "N")]
    reserved: UnsetField<u16>,
    /// Number of root entries for fat12 and fat16 variants.
    #[options(meta = "N")]
    root_entries: UnsetField<u16>,
    /// Power of two in the range[128,32768]. Typically 512 or 4096.
    #[options(meta = "N")]
    sector_size: UnsetField<u16>,
    /// Volume id.
    #[options(meta = "N")]
    volume_id: UnsetField<u32>,
    /// Profile to start with. One of {default,tiny,small,compat,large,huge}.
    #[options(parse(try_from_str="to_profile"))]
    profile: Profile,
}

#[derive(Debug, Default)]
enum Profile {
    #[default]
    Default,
    Tiny,
    Small,
    Compat,
    Large,
    Huge,
}

fn to_profile(s: &str) -> Result<Profile, &str> {
    match s {
        "default" => Ok(Profile::Default),
        "tiny" => Ok(Profile::Tiny),
        "small" => Ok(Profile::Small),
        "compat" => Ok(Profile::Compat),
        "large" => Ok(Profile::Large),
        "huge" => Ok(Profile::Huge),
        _ => Err(s),
    }
}

#[derive(PartialEq, Default, Debug)]
struct UnsetField<T>(Option<T>);

impl<T> core::ops::Deref for UnsetField<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Option<T> {
        &self.0
    }
}

impl<T: FromStr> FromStr for UnsetField<T> {
    type Err = T::Err;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(Self(Some(T::from_str(s)?)))
    }
}

/// Get a randomized volume ID.
fn rand_volume_id() -> u32 {
    let nsec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("no system time")
        .subsec_nanos();
    (nsec % 0xffff_ffff) + 1
}

fn main() -> Result<(), Error> {
    let opts = CommandOptions::parse_args_default_or_exit();

    let mut builder = match opts.profile {
        Profile::Default => MakeVFatFS::default(),
        Profile::Tiny => MakeVFatFS::tiny(),
        Profile::Small => MakeVFatFS::small(),
        Profile::Compat => MakeVFatFS::compat(),
        Profile::Large => MakeVFatFS::large(),
        Profile::Huge => MakeVFatFS::huge(),
    };
    builder.volume_id(rand_volume_id());
    // let the options override the parameters
    opts.align.map(|v| builder.align(v));
    opts.drive.map(|v| builder.drive(v));
    opts.label.as_ref().map(|v| builder.label(v));
    opts.media.map(|v| builder.media(v));
    opts.num_fats.map(|v| builder.num_fats(v));
    opts.oem.as_ref().map(|v| builder.oem(v));
    opts.reserved.map(|v| builder.reserved(v));
    opts.root_entries.map(|v| builder.root_entries(v));
    opts.volume_id.map(|v| builder.volume_id(v));
    if let Some(v) = *opts.per_cluster {
        builder.per_cluster(v)?;
    }
    if let Some(v) = *opts.sector_size {
        builder.sector_size(v)?;
    }

    if opts.verbose {
        println!("{builder:#?}");
    }

    let disk = LinuxDiskRW::new("/dev/stdin", opts.offset)?;
    let r = &disk as &dyn Read;
    let sectors = r.detect_size() / builder.get_sector_size() as u64;
    if opts.dry_run {
        builder.calc_variant(sectors)?;
        return Ok(())
    }
    builder.build(&disk, sectors)
}
