use std::sync::Once;
use tempfile::tempdir;

// size of a page in bytes
pub const PAGE_SIZE: usize = 4096;

// a page number that is always invalid
pub const INVALID_PAGE_NUM: isize = -1;

// max size of header page, each header page takes 2 bytes
pub const MAX_HEADER_PAGE: usize = PAGE_SIZE / 2;

// 1 bit per data page
pub const DATA_PAGES_PER_HEADER: usize = PAGE_SIZE * 8;

/// init for every test
///
/// _Note_: just for test
/// _Refer_: https://stackoverflow.com/questions/58006033/how-to-run-setup-code-before-any-tests-run-in-rust
const TEST_INIT: Once = Once::new();

pub fn initialize() {
    TEST_INIT.call_once(|| {
        let dir = tempdir().unwrap();
        let e = dir.as_ref().join("0").exists();
    })
}
