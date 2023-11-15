# storage.pico
The alpico storage stack.


## Supported FileSystems
- [x] ext4-ro
- [x] vfat-ro
- [x] json

## Library
- LinuxDisk
- InlineCache
- MemoryCache
- ReadSlice

## Tools

- [x] du
- [x] du-parallel
- [x] find
- [x] cat


## Todo
- [x] unified FS
  - [ ] introduce derive macro to specialize the implementation
- [x] partition support
  - [x] aliases
  - [x] recursive partitions
  - [ ] extended partitions
- [ ] metadata per file
  - [x] size
  - [x] id
  - [ ] ftype
  - [ ] mtime
  - [ ] btime
- [ ] memory cache
  - [ ] multiple ways
