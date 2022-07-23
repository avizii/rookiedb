use crate::common::constant::{DATA_PAGES_PER_HEADER, MAX_HEADER_PAGE, PAGE_SIZE};
use crate::common::Bit;
use crate::recovery::RecoveryManager;
use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, BytesMut};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::FileExt;
use std::sync::Mutex;

pub struct PartitionHandle {
    /// Underlying OS file
    file: Option<File>,
    /// Locks on the partition
    part_lock: Mutex<u8>,
    /// Contents of the master page of this partition
    master_page: Vec<u16>,
    /// Contents of the various header pages of this partition, actually represents like a `[[u8; 4096]; 2048]` array
    header_pages: Vec<Vec<u8>>,
    /// Partition number
    part_num: usize,
    /// Recovery manager
    /// TODO: type is missing
    recovery_manager: Box<dyn RecoveryManager>,
}

impl Drop for PartitionHandle {
    fn drop(&mut self) {
        println!("partition {} close.", self.part_num)
    }
}

impl PartitionHandle {
    pub fn new(part_num: usize, recovery_manager: Box<dyn RecoveryManager>) -> Self {
        Self {
            file: None,
            part_lock: Mutex::new(0),
            master_page: Vec::with_capacity(MAX_HEADER_PAGE),
            header_pages: Vec::with_capacity(MAX_HEADER_PAGE),
            part_num,
            recovery_manager,
        }
    }

    /// Opens the OS file and loads the master page and header pages.
    pub fn open(&mut self, file_name: String) -> Result<()> {
        self.file = Some(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(file_name)?,
        );

        // https://stackoverflow.com/questions/69738600/simplest-way-to-unwrap-an-option-and-return-error-if-none-anyhow

        match self.file {
            None => return Err(anyhow!("Could not open or read file")),
            Some(ref file) => {
                let length = file.metadata()?.len();
                if length == 0 {
                    // new file, write empty master page
                    self.write_master_page()
                } else {
                    // old file, read in master page + header pages
                    let mut buf = BytesMut::with_capacity(PAGE_SIZE);
                    file.read_at(buf.as_mut(), Self::master_page_offset() as u64)?;

                    for i in 0..MAX_HEADER_PAGE {
                        self.master_page.insert(i, buf.get_u16());
                        if Self::header_page_offset(i) < length as usize {
                            // load header page that were already in the file
                            let mut header_page: Vec<u8> = Vec::with_capacity(PAGE_SIZE);
                            file.read_at(
                                header_page.as_mut_slice(),
                                Self::header_page_offset(i) as u64,
                            );
                            self.header_pages.insert(i, header_page);
                        }
                    }

                    Ok(())
                }
            }
        }
    }

    /// Allocates a new page in the partition, and return the allocated DataPage number.
    pub fn alloc_page(&mut self) -> Result<usize> {
        let mut header_index = -1_isize;
        let mut page_index = -1_isize;

        // get free header page
        for i in 0..MAX_HEADER_PAGE {
            if let Some(header_page) = self.master_page.get(i) {
                if *header_page < DATA_PAGES_PER_HEADER as u16 {
                    header_index = i as isize;
                    break;
                }
            }
        }

        if header_index == -1 {
            return Err(anyhow!("no free pages - partition has reached max size"));
        }

        // get free data page
        match self.header_pages.get(header_index as usize) {
            None => {
                page_index = 0;
            }
            Some(header_content) => {
                for i in 0..DATA_PAGES_PER_HEADER {
                    if Bit::get_bit(header_content.as_slice(), i as u32)?.eq(&Bit::Zero) {
                        page_index = i as isize;
                        break;
                    }
                }

                if page_index == -1 {
                    return Err(anyhow!("header page should have free space, but doesn't"));
                }
            }
        }

        self.alloc_page_specific(header_index as usize, page_index as usize)
    }

    /// Allocates a new page in the partition, and return the allocated DataPage number.
    pub fn alloc_page_specific(&mut self, header_index: usize, page_index: usize) -> Result<usize> {
        let header_content: &mut Vec<u8> = match self.header_pages.get_mut(header_index) {
            None => {
                let header_content: Vec<u8> = Vec::with_capacity(PAGE_SIZE);
                self.header_pages.insert(header_index, header_content);

                // here cannot return `&mut header_content` directly, because the ownership of `header_content`
                // was moved into header_pages after `insert` operation.
                // for the moment, just get the reference from header_pages which stored in the header_index slot.
                unsafe { self.header_pages.get_unchecked_mut(header_index) }
            }
            Some(header_content) => header_content,
        };

        if Bit::get_bit(header_content, page_index as u32)?.eq(&Bit::One) {
            Err(anyhow!(
                "page at (partition={}, header={}, index={}) already allocated",
                self.part_num,
                header_index,
                page_index
            ))
        } else {
            Bit::set_bit(header_content.as_mut_slice(), page_index as u32, Bit::One)?;
            self.master_page
                .insert(header_index, Bit::count_ones(header_content) as u16);

            let page_num = page_index + header_index * DATA_PAGES_PER_HEADER;

            // TODO transaction and recovery manager
            // TransactionContext transaction = TransactionContext.getTransaction();
            // long vpn = DiskSpaceManager.getVirtualPageNum(partNum, pageNum);
            // if (transaction != null) {
            //     recoveryManager.logAllocPage(transaction.getTransNum(), vpn);
            // }
            // recoveryManager.diskIOHook(vpn);

            // flush the master page and header pages to Disk
            self.write_master_page();
            self.write_header_page(header_index)?;

            Ok(page_num)
        }
    }

    /// Reads in a DataPage. Assumes that the partition lock is held.
    pub fn read_page(&self, page_num: usize, buf: &mut [u8]) -> Result<()> {
        if self.is_not_allocated_page(page_num)? {
            Err(anyhow!("page {} is not allocated", page_num))
        } else {
            match self.file {
                None => Err(anyhow!("Could not open or read file")),
                Some(ref file) => {
                    file.read_at(buf, Self::data_page_offset(page_num) as u64)?;
                    Ok(())
                }
            }
        }
    }

    /// Writes to a DataPage. Assumes that the partition lock is held.
    pub fn write_page(&self, page_num: usize, buf: &[u8]) -> Result<()> {
        if self.is_not_allocated_page(page_num)? {
            Err(anyhow!("page {} is not allocated", page_num))
        } else {
            match self.file {
                None => Err(anyhow!("Could not open or read file")),
                Some(ref file) => {
                    file.write_at(buf, Self::data_page_offset(page_num) as u64)?;
                    // force sync the data without metadata info to disk
                    file.sync_data()?;

                    // TODO
                    // long vpn = DiskSpaceManager.getVirtualPageNum(partNum, pageNum);
                    // recoveryManager.diskIOHook(vpn);

                    Ok(())
                }
            }
        }
    }

    /// Writes the master page to disk.
    fn write_master_page(&self) -> Result<()> {
        let mut buf = BytesMut::with_capacity(PAGE_SIZE);
        self.master_page.iter().for_each(|v| buf.put_u16(*v));
        match self.file {
            None => Err(anyhow!("Could not open or read file")),
            Some(ref file) => {
                file.write_at(buf.as_ref(), Self::master_page_offset() as u64)?;
                Ok(())
            }
        }
    }

    /// Writes a header page to disk.
    fn write_header_page(&self, header_index: usize) -> Result<()> {
        if let Some(header_page) = self.header_pages.get(header_index) {
            match self.file {
                None => return Err(anyhow!("Could not open or read file")),
                Some(ref file) => {
                    file.write_at(header_page, Self::header_page_offset(header_index) as u64)?;
                }
            }
        }
        Ok(())
    }

    /// Frees a DataPage in the partition from used.
    pub fn free_page(&mut self, page_num: usize) -> Result<()> {
        let header_index = page_num / DATA_PAGES_PER_HEADER;
        let page_index = page_num % DATA_PAGES_PER_HEADER;

        match self.header_pages.get_mut(header_index) {
            None => Err(anyhow!("cannot free unallocated page")),
            Some(header_content) => {
                if Bit::get_bit(header_content.as_slice(), page_index as u32)?.eq(&Bit::Zero) {
                    Err(anyhow!("cannot free unallocated page"))
                } else {
                    // TODO Transaction and RecoveryManager
                    // TransactionContext transaction = TransactionContext.getTransaction();
                    // long vpn = DiskSpaceManager.getVirtualPageNum(partNum, pageNum);
                    // if (transaction != null) {
                    //     byte[] contents = new byte[PAGE_SIZE];
                    //     readPage(pageNum, contents);
                    //     int halfway = BufferManager.RESERVED_SPACE + BufferManager.EFFECTIVE_PAGE_SIZE / 2;
                    //     recoveryManager.logPageWrite(
                    //         transaction.getTransNum(),
                    //         vpn,
                    //         (short) 0,
                    //         Arrays.copyOfRange(contents, BufferManager.RESERVED_SPACE, halfway),
                    //         new byte[BufferManager.EFFECTIVE_PAGE_SIZE / 2]
                    //     );
                    //     recoveryManager.logPageWrite(
                    //         transaction.getTransNum(),
                    //         vpn,
                    //         (short) (BufferManager.EFFECTIVE_PAGE_SIZE / 2),
                    //         Arrays.copyOfRange(contents, halfway, PAGE_SIZE),
                    //         new byte[BufferManager.EFFECTIVE_PAGE_SIZE / 2]
                    //     );
                    //     recoveryManager.logFreePage(transaction.getTransNum(), vpn);
                    // }
                    // recoveryManager.diskIOHook(vpn);

                    Bit::set_bit(header_content.as_mut_slice(), page_index as u32, Bit::Zero)?;
                    self.master_page.insert(
                        header_index,
                        Bit::count_ones(header_content.as_slice()) as u16,
                    );
                    self.write_master_page()?;
                    self.write_header_page(header_index)?;
                    Ok(())
                }
            }
        }
    }

    /// Frees all DataPages from partition for used.
    pub fn free_data_pages(&mut self) -> Result<()> {
        let mut v = vec![];
        for i in 0..MAX_HEADER_PAGE {
            if let Some(v) = self.master_page.get(i) {
                if *v <= 0 {
                    continue;
                }
            }

            let header_content = match self.header_pages.get(i) {
                None => continue,
                Some(header_content) => header_content,
            };

            for j in 0..DATA_PAGES_PER_HEADER {
                if Bit::get_bit(header_content.as_slice(), j as u32)?.eq(&Bit::One) {
                    // here cannot call `self.free_page()` directly which cannot borrow `*self` as mutable
                    // because it is also borrowed as immutable.
                    // the solution is cache the header index and page index in vec,
                    // and then call `self.free_page()` individually after the loop.
                    v.push((i, j));
                }
            }
        }

        for (header_index, page_index) in v.iter() {
            self.free_page(header_index * DATA_PAGES_PER_HEADER + page_index)?;
        }

        Ok(())
    }

    /// Checks if page number is for an unallocated data page
    pub fn is_not_allocated_page(&self, page_num: usize) -> Result<bool> {
        let header_index = page_num / DATA_PAGES_PER_HEADER;
        let page_index = page_num % DATA_PAGES_PER_HEADER;

        if header_index >= MAX_HEADER_PAGE {
            return Ok(true);
        }

        if let Some(v) = self.master_page.get(header_index) {
            if *v == 0 {
                return Ok(true);
            }
        }

        if let Some(v) = self.header_pages.get(header_index) {
            return Ok(Bit::get_bit(v.as_slice(), page_index as u32)?.eq(&Bit::Zero));
        }

        Ok(false)
    }

    /// Returns the offset in OS file for master page.
    fn master_page_offset() -> usize {
        0
    }

    /// Returns the offset in OS file for specific header page.
    ///
    /// # Example
    ///
    /// Consider the layout if we had 4 data pages per header:
    /// Offset(in pages):  0   1   2   3   4   5   6   7   8   9  10  11
    /// Page Type:        [M] [H] [D] [D] [D] [D] [H] [D] [D] [D] [D] [H]...
    /// Header Index:          0                   1                   2
    ///
    /// To get the offset in pages of a header page, you should add 1 for the master page,
    /// and then take the header index times the number of of data pages per header plus 1
    /// to account for the header page itself.
    /// (in the above example this coefficient would be 5)
    fn header_page_offset(header_index: usize) -> usize {
        // plus the self header page every one round
        // then plus the single master page
        (1 + (DATA_PAGES_PER_HEADER + 1) * header_index) * PAGE_SIZE
    }

    /// Returns the offset in OS file for specific data page.
    ///
    /// # Example
    ///
    /// Consider the layout if we had 4 data pages per header:
    /// Offset(in pages):  0   1   2   3   4   5   6   7   8   9  10  11
    /// Page Type:        [M] [H] [D] [D] [D] [D] [H] [D] [D] [D] [D] [H]...
    /// Header Index:              0   1   2   3       4   5   6   7
    ///
    /// To get the offset in pages of a given data page. you should:
    /// - add one for the master page
    /// - add one for the first header page
    /// - add how many other header pages precede the data page(found by floor dividing page num by data pages per header)
    /// - add how many data pages precede the given data page(this works out conveniently to the page's page number)
    fn data_page_offset(page_num: usize) -> usize {
        let previous_headers = page_num / DATA_PAGES_PER_HEADER;
        // master page + first header + other headers + current page num
        (1 + 1 + previous_headers + page_num) * PAGE_SIZE
    }
}
