//! Directory iteration for vfat.

use super::{file::File, DirectoryEntry, Error, Offset};
use ap_storage::{directory, meta::FileType, Read, ReadExt};

pub struct Dir<'a> {
    file: &'a File<'a>,
    offset: Offset,
}

impl<'a> Dir<'a> {
    pub(crate) fn new(file: &'a File<'a>) -> Self {
        Self { file, offset: 0 }
    }

    /// Return an absolute directory entry.
    fn get_abs(&self, abs: u64) -> Result<DirectoryEntry, Error> {
        if !self.file.is_root() {
            (self.file as &dyn Read).read_object(abs * 32)
        } else {
            // the root directory does not have self-pointers - fabricate them
            match abs {
                0 => Ok(self.file.inode),
                1 => Ok(self.file.inode),
                _ => (self.file as &dyn Read).read_object(abs * 32 - 64),
            }
        }
    }

    /// Return the next directory entry.
    fn get_next(&mut self) -> Result<DirectoryEntry, Error> {
        let res = self.get_abs(self.offset)?;
        self.offset += 1;
        Ok(res)
    }

    /// Retrieve the long-name from the entries and put it into the name.
    #[cfg(feature="long-name")]
    fn handle_long_name(&self, next_offset: u64, name: &mut [u8]) -> usize {
        let mut res = 0;
        // look at the long-entries
        let long_count = self.offset - next_offset;
        for i in 0..long_count {
            let Ok(e) = self.get_abs(self.offset - 2 - i) else {
                return 0;
            };
            let lentry: ap_storage_vfat::LongEntry = unsafe { core::mem::transmute(e) };

            for ch in char::decode_utf16(lentry.name()) {
                let mut buf = [0u8; 4];
                let r = ch.unwrap_or(char::REPLACEMENT_CHARACTER).encode_utf8(&mut buf);
                if res + r.len() > name.len() {
                    return res;
                }
                name[res..res + r.len()].copy_from_slice(r.as_bytes());
                res += r.len();
            }
        }
        res
    }

    /// Detect a longname and return the real entry.
    #[cfg(feature="long-name")]
    fn detect_longname(&mut self) -> Result<(DirectoryEntry, Offset), Error> {
        let mut entry = self.get_next()?;
        let mut next_offset = self.offset;
        let mut long_entries = 0;
        while entry.attr & 0x3f == 0xf {
            let lentry: ap_storage_vfat::LongEntry = unsafe { core::mem::transmute(entry) };
            // combine different fields into one value to simplify the valid checks
            let x = ((lentry.typ as u32) << 16) | ((lentry.ord as u32) << 8) | lentry.cksum as u32;
            if x & 0x4000 != 0 {
                // start of a novel entry - there can be multiple ones - skip the previous entries
                next_offset = self.offset;
                long_entries = x;
            }

            // non-continous entry, to-small or to much entries
            if (x | 0x4000) != long_entries || !(0x4100..0x4000 + (21 << 8)).contains(&long_entries) {
                // signal that we skip these number of entries
                next_offset = self.offset;
                break;
            }

            long_entries -= 0x100;
            entry = self.get_next()?;
        }
        // compare the checksum to figure out whether the long-values fit the entry
        if long_entries != 0 && entry.checksum() != (long_entries & 0xff) as u8 {
            next_offset = self.offset;
        }
        Ok((entry, next_offset))
    }
}

impl<'a> directory::Iterator for Dir<'a> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<directory::Item>, Error> {
        #[cfg(not(feature="long-name"))]
        let entry = self.get_next()?;
        #[cfg(feature="long-name")]
        let (entry, next_offset) = self.detect_longname()?;

        // end-of-directory?
        if entry.name[0] == 0 {
            return Ok(None);
        }
        let typ = if entry.attr & 0x8 != 0 || entry.name[0] == 0xe5 {
            FileType::Unknown
        } else if self.offset < 3 {
            FileType::Parent
        } else if entry.is_dir() {
            FileType::Directory
        } else {
            FileType::File
        };

        // get the long-name if present
        let mut nlen = 0;
        #[cfg(not(feature="ignore-long-name"))]
        if !self.file.fs.options.ignore_long_name {
            nlen = self.handle_long_name(next_offset, name)
        };

        // take the short-name if no long-name was found.
        if nlen == 0 {
            let mut shortname = entry.name();
            nlen = shortname.trim_ascii_end().len();

            // convert to lower-case
            if self.file.fs.options.lower_short_name {
                for x in shortname.iter_mut() {
                    *x = x.to_ascii_lowercase();
                }
            }
            if self.offset == 1 && self.file.is_root() {
                // drop one dot from the first pointer
                nlen = 1;
            }
            let n = core::cmp::min(nlen, name.len());
            name[..n].copy_from_slice(&shortname[..n]);
        }

        Ok(Some(directory::Item {
            offset: self.offset - 1,
            nlen,
            typ,
            id: entry.cluster() as u64,
        }))
    }
}
