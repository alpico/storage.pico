# The alpico Storage Stack

#### A modular and portable approach to persisting data

[![storage.pico logo](.logo.png)](https://github.com/alpico/storage.pico)

## The modular approach

alpico is developing an ecosystem for embedded software development.
We believe that by using a modular approach, software can be made a
lot smaller and a lot less complex offering many benefits to embedded
devices - both in performance and ease of development.

To facilitate modularity and ensure the smallest possible footprint in
every use case, we divide functionality strongly between crates.
Crates, in turn, use feature flags, so you only ever use what *you*
need.

Read-only and read-write versions of filesystems will commonly be
split into two crates because write functionality often requires to
save additional information in structs compared to read-only.
Generating (`mkfs`) and checking (`fsck`) are not required at all for
normal interaction with a filesystem and are therefore separate crates
as well.  Finally, there will be end-to-end integration tests in their
own crates.



## Portability

All crates in this repo, except for the examples in [`ap-storage-examples`](./creates/ap-storage-examples), are `#![no_std]` compatible.

## Supported Filesystems

- [ext4-ro](./crates/ap-storage-ext4-ro/)
- [json](./crates/ap-storage-json/)
- [partitions](./crates/ap-storage-partition/)
- [vfat-ro](./crates/ap-storage-vfat-ro/)

## Utilities

- [LinuxDisk](./crates/ap-storage-linux/)
- [InlineCache](./crates/ap-storage-memory/)
- [ReadSlice](./crates/ap-storage-memory/)
- [date](./crates/ap-date/)

## Examples

We have examples of the implementation of tools using our storage interface.

- [cat](./crates/ap-storage-examples/src/bin/cat.rs)
- [du-parallel](./crates/ap-storage-examples/src/bin/du-parallel.rs)
- [du](./crates/ap-storage-examples/src/bin/du.rs)
- [find](./crates/ap-storage-examples/src/bin/find.rs)
- [mkfs-vfat](./crates/ap-storage-examples/src/bin/mkfs-vfat.rs)

## Roadmap

- [ ] pseudo disks
  - [ ] /dev/zero - read returns nul-bytes, writes are ok
  - [ ] concat aka RAID0
- [x] unified FS - Use a single struct for interacting with any supported filesystem
  - [ ] introduce derive macro to specialize the implementation
- [x] partition support
  - [ ] follow extended/logical partitions
- [ ] external memory cache
  - [ ] support multiple ways
