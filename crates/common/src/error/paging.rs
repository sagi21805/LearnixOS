// ANCHOR: use_thiserror
use thiserror::Error;
// ANCHOR_END: use_thiserror

#[derive(Error, Debug)]
pub enum TableError {
    #[error("The table is not a root table")]
    NotRoot,
    #[error("The table is full")]
    Full,
}

// ANCHOR: entry_error
#[derive(Error, Debug)]
pub enum EntryError {
    #[error("There is no mapping to this entry")]
    NoMapping,
    #[error("This entry contains memory block and not a table")]
    NotATable,
}
// ANCHOR_END: entry_error
