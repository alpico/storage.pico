# ap-storage-ext4

#### This crate is part of

[![storage.pico logo](../../.logo.png)](https://github.com/alpico/storage.pico)

---

A read-only implementation of ext4 for the alpico storage stack.

## Usage

```rust
use ap_storage_ext4_ro::Ext4Fs;
use ap_storage::{FileSystem, file::File, Read};

let fs = Ext4Fs::new(&disk, false)?;
let root = fs.root()?;
let file = root.lookup(b"file.txt")?.expect("File not found");
let mut read_buf = [0u8; 32];
let len = file.read_bytes(0, &mut read_buf)?;
let output = std::str::from_utf8(&read_buf[..len])?;
println!("{output}");
```
