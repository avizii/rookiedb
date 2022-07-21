type LockContext = u32;
type BufferFrame = u32;

pub struct Page {
    pub lock_context: LockContext,
    pub frame: BufferFrame,
}

type BufferManager = u32;
type HeaderPage = u32;

pub struct PageDirectory {
    pub buffer_manager: BufferManager,
    pub part_num: usize,
    pub first_header: HeaderPage,
    pub empty_page_metadata_size: i16,
    pub lock_context: LockContext,
    pub page_directory_id: usize,
}
