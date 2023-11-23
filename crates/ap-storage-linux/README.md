# ap-storage-linux

#### This crate is part of

[![storage.pico logo](../../.logo.png)](https://github.com/alpico/storage.pico)

---

A read-only adapter to POSIX files for the alpico storage stack.

Not that this crate is __not__ `#![no_std]` compatible.

## Usage

```rust
use ap_storage_linux::LinuxDisk;
use ap_storage::Read;

let file = LinuxDisk::new("file.txt", 0);
let mut read_buf = [0u8; 32];
let len = file.read_bytes(0, &mut read_buf)?;
let output = std::str::from_utf8(&read_buf[..len])?;
println!("{output}");
```
