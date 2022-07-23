use crate::io::partition::PartitionHandle;
use std::collections::HashMap;

trait StorageManager {
    /// Allocates a new partition.
    fn alloc_part(&mut self) -> usize;

    /// Allocates a new partition with a specific partition number.
    fn alloc_part_specific(&mut self, part_num: usize) -> usize;

    /// Releases a partition from used.
    fn free_part(&mut self, part_num: usize);

    /// Allocates a new page and partition to allocate new page under.
    fn alloc_page_from_part(&mut self, part_num: usize) -> usize;

    /// Allocates a new page with a specific page number.
    fn alloc_page(&mut self, page_num: usize) -> usize;

    /// Frees a page.
    ///
    /// _Note_: The page cannot be used after this call.
    fn free_page(&mut self, page: usize);

    /// Reads a page to a byte buffer whose contents will be filled with page data.
    fn read_page(&mut self, page: usize, buf: Vec<u8>);

    /// Writes to a page.
    fn write_page(&mut self, page: usize, buf: Vec<u8>);

    /// Checks if a page is allocated.
    fn page_allocated(&mut self, page: usize) -> bool;

    /// TODO implement Drop Trait
    fn close(&self);

    /// Gets partition number from virtual page number.
    fn get_part_num(page: usize) -> usize {
        (page / 10000000000) as usize
    }

    /// Gets data page number from virtual page number.
    fn get_page_num(page: usize) -> usize {
        (page % 10000000000) as usize
    }

    /// Gets the virtual page number by given partition/data page number.
    fn get_virtual_page_num(part_num: usize, page_num: usize) -> usize {
        part_num * 10000000000 + page_num
    }
}

pub struct DiskSpaceManager {
    /// Name of base directory
    db_dir: String,
    /// Information about each partition
    part_info: PartitionHandle,
}

impl StorageManager for DiskSpaceManager {
    fn alloc_part(&mut self) -> usize {
        todo!()
    }

    fn alloc_part_specific(&mut self, part_num: usize) -> usize {
        todo!()
    }

    fn free_part(&mut self, part_num: usize) {
        todo!()
    }

    fn alloc_page_from_part(&mut self, part_num: usize) -> usize {
        todo!()
    }

    fn alloc_page(&mut self, page_num: usize) -> usize {
        todo!()
    }

    fn free_page(&mut self, page: usize) {
        todo!()
    }

    fn read_page(&mut self, page: usize, buf: Vec<u8>) {
        todo!()
    }

    fn write_page(&mut self, page: usize, buf: Vec<u8>) {
        todo!()
    }

    fn page_allocated(&mut self, page: usize) -> bool {
        todo!()
    }

    fn close(&self) {
        todo!()
    }
}

/*#[cfg(test)]
mod tests {
    use crate::common::constant;
    use crate::storage::{DiskSpaceManager, StorageManager};
    use std::fs::File;
    use tempfile::TempDir;

    fn get_disk_space_manager() -> (DiskSpaceManager, TempDir) {
        todo!()
    }

    #[test]
    fn test_create_disk_space_manager() {
        let (dsm, _dir) = get_disk_space_manager();
        dsm.close()
    }

    #[test]
    fn test_alloc_part() {
        let (mut dsm, dir) = get_disk_space_manager();

        let part_num = dsm.alloc_part_specific(0);

        assert_eq!(0, part_num);
        assert!(dir.path().join("0").exists());

        // _Google_: [how to get file length in rust](https://stackoverflow.com/questions/54303398/how-to-get-the-size-of-an-already-opened-file-in-rust)
        assert_eq!(
            constant::PAGE_SIZE as u64,
            File::open(dir.path().join("0"))
                .unwrap()
                .metadata()
                .unwrap()
                .len()
        );

        let part_num = dsm.alloc_part();

        assert_eq!(1, part_num);
        assert!(dir.path().join("1").exists());
        assert_eq!(
            constant::PAGE_SIZE as u64,
            File::open(dir.path().join("1"))
                .unwrap()
                .metadata()
                .unwrap()
                .len()
        );

        dsm.close();
    }

    #[test]
    fn test_alloc_part_persist() {
        let (mut dsm, dir) = get_disk_space_manager();

        dsm.alloc_part();
        dsm.close();
    }
}*/
