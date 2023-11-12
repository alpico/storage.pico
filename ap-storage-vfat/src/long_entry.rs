//! Long directory entries.

/// A long directory entry.
#[derive(Clone, Copy, Default, Debug)]
#[repr(packed)]
pub struct LongEntry {
    pub ord: u8,
    pub name1: [u16; 5],
    pub attr: u8,
    pub typ: u8,
    pub cksum: u8,
    pub name2: [u16; 6],
    pub _x: u16,
    pub name3: [u16; 2],
}

impl LongEntry {
    pub fn name(&self) -> LongEntryIter {
        LongEntryIter::new(self)
    }
}

/// An iterator over the name fields in a long-entry.
pub struct LongEntryIter<'a> {
    v: &'a LongEntry,
    pos: usize,
}

impl<'a> LongEntryIter<'a> {
    pub fn new(v: &'a LongEntry) -> Self {
        Self { v, pos: 0 }
    }
}

impl Iterator for LongEntryIter<'_> {
    type Item = u16;
    fn next(&mut self) -> Option<u16> {
        let v = match self.pos {
            0..=4 => self.v.name1[self.pos],
            5..=10 => self.v.name2[self.pos - 5],
            11..=12 => self.v.name3[self.pos - 11],
            _ => return None,
        };
        if v == 0 {
            return None;
        }
        self.pos += 1;
        Some(v)
    }
}
