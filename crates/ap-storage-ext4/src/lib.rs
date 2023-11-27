//! On-disk structures for Ext{2,3,4}.

#![no_std]
// This crate contains on-disk structures that are already defined in various specifications.
// There is no need to copy-paste their docs here.
#![allow(missing_docs)]

pub mod dir;
pub mod extent;
pub mod inode;
pub mod superblock;
