use crate::databox::DataBox;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DBError {
    #[error("{0}")]
    IllegalArgumentError(&'static str),

    #[error("Not {1} databox: {:0}")]
    TypeError(DataBox, &'static str),

    #[error("Get bit in byte: index {0} out of bounds")]
    BitOutBoundError(u32),
}
