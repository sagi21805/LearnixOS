use thiserror::Error;

#[derive(Error, Debug)]
pub enum TableError {
    #[error("The table is not a root table")]
    NotRoot,
    #[error("The table is full")]
    Full,
}

#[derive(Error, Debug)]
pub enum EntryError {
    #[error("There is no mapping to this entry")]
    NoMapping(usize),
    #[error("This entry contains memory block and not a table")]
    NotATable(usize),
    #[error("Can't provide another entry, the table is full")]
    Full,
}
