//! Support for object attributes.
//!
//! There are various places in alpico where meta-data needs to be made publicly available.
//! Examples are:
//!   - file attributes like `SIZE` or  `MTIME`.
//!   - file-system attributes like `BLOCKSIZE` or `ID`.
//!   - driver config like an UART baud rate
//!
//! ### Naming
//!
//! The attribute name-space is global.  To avoid conflicts we prefix
//! the name with the crate it is defined in.
//!
//! ### Types
//!
//! - there are signed and unsigned 64-bit values
//! - there are boolean and raw-byte values
//! - times are defined as nano-seconds since UNIX_EPOCH.

#![no_std]

mod value;
pub use value::Value;

/// Define an attribute name.
#[macro_export]
macro_rules! new_attr {
    ($name: ident, $typ:ident, $doc:literal) => {
        #[doc=concat!($doc, " (", stringify!($typ), ")")]
        pub const $name: &str = concat!(env!("CARGO_PKG_NAME"), ".", stringify!($name));
    };
}

/// Trait for reading attributes.
///
/// The keys can be iterated.
pub trait Attributes<'a>: IntoIterator<Item = &'a &'a str> {
    /// Get a value.  The (optional) buffer is used when raw-values are returned.
    fn get(&self, name: &str, buf: &mut [u8]) -> Option<Value>;
}
