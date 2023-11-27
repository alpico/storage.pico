//! Support for extended attributes in files.

pub use ap_util_attr::{new_attr, Attributes, Value};

new_attr!(ATIME, I64, "Time of last file access.");
new_attr!(BTIME, I64, "Time of file birth.");
new_attr!(FTYPE, Raw, "File type.");
new_attr!(ID, U64, "A unique ID of the file, used to detect hard-links.");
new_attr!(MTIME, I64, "Time of last file modification.");
new_attr!(SIZE, U64, "The size of the file in bytes.");
