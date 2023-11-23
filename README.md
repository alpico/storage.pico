# The alpico Storage Stack

#### A modular and portable approach to persisting data

[![storage.pico logo](.logo.png)](https://github.com/alpico/storage.pico)

## The modular approach

alpico is developing an ecosystem for embedded software development.
We believe that by using a modular approach, software can be made a lot smaller and a lot less complex
offering many benefits to embedded devices - both in performance and ease of development.

To facilitate modularity and ensure the smallest possible footprint in every use case, we divide functionality strongly between crates.
Crates, in turn, heavily use feature flags, so you only ever use what *you* need.

Read-only and read-write versions of filesystems will commonly be split into two crates because write functionality often requires to save additional information in structs compared to read-only.
Mkfs and fsck are not required at all for normal interaction with a filesystem and are therefore separate crates.

## Portability

All crates in this repo, except for [`ap-storage-linux`](./creates/ap-storage-linux), are `#![no_std]` compatible.

## Supported Filesystems

- [ext4-ro](./crates/ap-storage-ext4-ro/)
- [vfat-ro](./crates/ap-storage-vfat-ro/)
- [json](./crates/ap-storage-json/)
- [partitions](./crates/ap-storage-partition/)

## Utilities

- [LinuxDisk](./crates/ap-storage-linux/)
- [InlineCache](./crates/ap-storage-memory/)
- [ReadSlice](./crates/ap-storage-memory/)
- [date](./crates/ap-date/)

## Examples

We have examples of the implementation of common tools using our storage interface.

- [du](./crates/ap-storage-linux/examples/du.rs)
- [du-parrallel](./crates/ap-storage-linux/examples/du-parrallel.rs)
- [find](./crates/ap-storage-linux/examples/find.rs)
- [cat](./crates/ap-storage-linux/examples/cat.rs)

## Roadmap

- [x] unified FS - Use a single struct for interacting with any supported filesystem
  - [ ] introduce derive macro to specialize the implementation
- [x] partition support
  - [ ] follow extended/logical partitions
- [ ] extensible metadata
- [ ] external memory cache
  - [ ] support multiple ways
