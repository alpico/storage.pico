use ap_storage::{Error, Offset, Read};

/// A memory cache to speedup reads to an underlying disk.
///
/// TODO: LRU linked list, Multiple CacheSets
pub struct MemoryCacheImpl<'a> {
    /// The disk to cache the data for.
    parent: &'a dyn Read,
    /// The pages of user-data.
    userdata: &'a mut [u8],
    /// The metadata per page.
    meta: &'a mut [Metadata],
    /// The last-recently used entry for a simple round-robing strategy.
    lru: usize,
}

impl<'a> MemoryCacheImpl<'a> {
    const PAGE_SIZE: usize = 4096;

    /// The data is used as backing store.
    pub fn new(data: &'a mut [u8], parent: &'a dyn Read) -> Self {
        let meta_size = core::mem::size_of::<Metadata>();
        let pages = data.len() / (Self::PAGE_SIZE + meta_size);

        // we seperate the backing store here - the first is used for the pages, the remaining for the metadata
        let split = pages * Self::PAGE_SIZE;
        let meta = unsafe {
            core::slice::from_raw_parts_mut(data.as_mut_ptr().add(split) as *mut Metadata, pages)
        };
        let userdata = &mut data[..split];

        // initialize the metadata to point to nothing
        for entry in meta.iter_mut() {
            entry.page_offset = !0;
        }

        Self {
            parent,
            userdata,
            meta,
            lru: 0,
        }
    }

    /// Read from the disk. This requires a mutable self.
    pub fn read_mut(&mut self, ofs: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let page_offset = ofs / Self::PAGE_SIZE as Offset;
        let in_page = (ofs % Self::PAGE_SIZE as Offset) as usize;
        let maxn = core::cmp::min(Self::PAGE_SIZE - in_page, buf.len());
        let pages = self.meta.len();

        // search in the cache
        for i in 0..pages {
            let index = (self.lru + i) % pages;
            if self.meta[index].page_offset == page_offset {
                let pofs = index * Self::PAGE_SIZE + in_page;
                buf[..maxn].copy_from_slice(&self.userdata[pofs..pofs + maxn]);
                return Ok(maxn);
            }
        }

        // advance the LRU pointer
        self.lru = (self.lru + 1) % pages;

        // invalidate the entry as we might destroy the data in it
        self.meta[self.lru].page_offset = !0;
        let our = &mut self.userdata[self.lru * Self::PAGE_SIZE..(self.lru + 1) * Self::PAGE_SIZE];

        // try to read a full page
        let mut n = 0;
        while n != Self::PAGE_SIZE {
            let x = self
                .parent
                .read_bytes(ofs - (in_page + n) as Offset, &mut our[n..]);
            match x {
                Err(_) if n > in_page => break,
                Err(e) => return Err(e),
                Ok(0) => break,
                Ok(c) => {
                    n += c;
                }
            }
        }

        // only enable the cache if the whole page is filled
        if n == Self::PAGE_SIZE {
            self.meta[self.lru].page_offset = page_offset;
        }
        let maxn = core::cmp::min(maxn, n - in_page);
        buf[..maxn].copy_from_slice(&our[in_page..in_page + maxn]);
        Ok(maxn)
    }
}

/// The metadata for a single entry in the cache.
#[repr(C)]
struct Metadata {
    /// The page offset.
    page_offset: Offset,
}
