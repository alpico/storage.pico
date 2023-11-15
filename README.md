# storage.pico
The alpico storage stack.


## Supported FileSystems
- [x] ext4-ro
- [x] vfat-ro
- [x] json
- [x] partitions

## Library
- LinuxDisk
- InlineCache
- MemoryCache
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
  - [ ] extended partitions
- [x] metadata per file
  - [x] size
  - [x] id
  - [x] ftype
  - [x] mtime
- [ ] extensible metadata
- [ ] memory cache
  - [ ] multiple ways
