# storage.pico

The alpico storage stack.

It consists of file-systems, block-devices and caching layer.


## Supported FileSystems
- [x] ext4-ro
- [x] vfat-ro
- [x] json
- [x] partitions

## Library
- LinuxDisk
- InlineCache
- ReadSlice
- date

## Tools
- [x] du
- [x] du-parallel
- [x] find
- [x] cat


## Todo
- [x] unified FS
  - [ ] introduce derive macro to specialize the implementation
- [x] partition support
  - [ ] follow extended/logical partitions
- [ ] extensible metadata
- [ ] external memory cache
  - [ ] support multiple ways
