//! Definition of file extents.


#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Ext4ExtentHeader {
    pub magic: u16,
    pub entries: u16,
    _0: u16,
    pub depth: u16,
    _1: u32,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Ext4ExtentLeaf {
    pub block: u32,
    pub len: u16,
    pub hi: u16,
    pub lo: u32,
}

impl Ext4ExtentLeaf {
    pub fn dest(&self) -> u64 {
        ((self.hi as u64) << 32) | self.lo as u64
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Ext4ExtentIndex {
    pub block: u32,
    pub lo: u32,
    pub hi: u16,
    _0: u16,
}

impl Ext4ExtentIndex {
    pub fn dest(&self) -> u64 {
        ((self.hi as u64) << 32) | self.lo as u64
    }
}
