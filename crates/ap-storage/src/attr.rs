//! Support for extended attributes.
//!
//! # Naming
//!
//! The attribute name-space is global.  To avoid conflicts we prefix
//! the name with the crate it is defined in.
//!
//! # Types
//!
//! - there are signed and unsigned 64-bit values
//! - there a
//! - times are defined as nano-seconds since UNIX_EPOCH.

pub use ap_util_attr::{Attributes, Value, new_attr};

new_attr!(ATIME, I64, "Time of last file access.");
new_attr!(BTIME, I64, "Time of file birth.");
new_attr!(FTYPE, Raw, "File type.");
new_attr!(ID, U64, "A unique ID of the file, used to detect hard-links.");
new_attr!(MTIME, I64, "Time of last file modification.");
new_attr!(SIZE, U64, "The size of the file in bytes.");

