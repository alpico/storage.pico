//! End-to-end tests for ap-storage-vfat-* crates.

#[cfg(test)]
mod tests {
    use ap_storage_vfat::Variant;
    use ap_storage_vfat_mkfs::MakeVFatFS;

    /// Validate that the calculated FAT sizes cover the whole fat
    #[test]
    fn mkfs_fat_size() {
        for num_fat in [1, 2, 3] {
            for per_cluster in [1, 2, 4] {
                let builder = MakeVFatFS::small().per_cluster(per_cluster).unwrap().num_fats(num_fat);
                let sector_size = builder.get_sector_size() as u64;

                // this covers all clusters in Fat12,Fat16 and the beginning of Fat32.
                let min_size = (2 + num_fat as u64).div_ceil(per_cluster as u64) + 1;
                for clusters in min_size..99999 {
                    let sectors = clusters * per_cluster as u64;
                    let (variant, fat_size) = builder.calc_variant(sectors).unwrap();
                    let data_start = builder.data_start(variant, fat_size);
                    let entries = fat_size * sector_size * 8 / (variant as u64);
                    let entries_per_sector = (sector_size * 8).div_ceil(variant as u64);

                    // this also checks for tightness
                    assert!(
                        entries < (sectors - data_start) / (per_cluster as u64) + entries_per_sector + 6,
                        "{entries} {sectors} {data_start} {variant:?} {fat_size} {per_cluster} {num_fat}"
                    )
                }
            }
        }
    }

    //
    #[test]
    fn mkfs_fat_minimal_size() {
        let mut builder = MakeVFatFS::small();
        for per_cluster in [1, 2, 4, 8] {
            builder.per_cluster(per_cluster).unwrap();
            let min_size = 3 + per_cluster as u64;
            assert!(builder.calc_variant(min_size).is_ok(), "{min_size} {per_cluster}");
            assert!(builder.calc_variant(min_size - 1).is_err(), "{min_size} {per_cluster}");
        }
    }

    #[test]
    fn mkfs_fat_compat_size() {
        let builder = MakeVFatFS::compat();
        for sectors in [0xfff_fffc, 0xffff_f800] {
            let (variant, fat_size) = builder.calc_variant(sectors).unwrap();
            assert_eq!(Variant::Fat32, variant);
            let data_start = builder.data_start(variant, fat_size);
            let clusters = (sectors - data_start) / builder.get_per_cluster() as u64 + 2;
            assert!(clusters < fat_size * builder.get_sector_size() as u64 * 8 / (variant as u64));
        }
    }
    #[test]
    fn mkfs_fat_size_max() {
        let mut builder = MakeVFatFS::small().sector_size(4096).unwrap();

        for order in 0..7 {
            builder.per_cluster(1 << order).unwrap();
            let limit = (0xffffff5 << order) + (1 << 30) / builder.get_sector_size() as u64 + (1 << order);
            assert!(builder.calc_variant(limit - 1).is_ok(), "{order} {limit:x}");
            assert!(builder.calc_variant(limit).is_err(), "{order} {limit:x}");
        }
    }
}
