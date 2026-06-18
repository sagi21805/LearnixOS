use macros::bitfields;

/// A wrapper for `PageTableEntry` flags for easier use
#[bitfields]
pub struct PageEntryFlags {
    pub present: B1,
    pub writable: B1,
    pub usr_access: B1,
    pub write_through_cache: B1,
    pub disable_cache: B1,
    #[flag(r)]
    pub accessed: B1,
    #[flag(r)]
    pub dirty: B1,
    pub huge_page: B1,
    pub global: B1,
    pub full: B1,
    pub table: B1,
    pub root_entry: B1,
}

impl PageEntryFlags {
    /// Default flags for entry that contains page table.
    pub const fn table_flags() -> Self {
        PageEntryFlags::new()
            .present(true)
            .writable(true)
            .table(true)
    }

    /// Default flags for entry that contains huge page.
    pub const fn huge_page_flags() -> Self {
        PageEntryFlags::new()
            .present(true)
            .writable(true)
            .huge_page(true)
    }

    /// Default flags for entry that contains regular page.
    pub const fn regular_page_flags() -> Self {
        PageEntryFlags::new().present(true).writable(true)
    }

    pub const fn regular_io_page_flags() -> Self {
        PageEntryFlags::new()
            .present(true)
            .writable(true)
            .disable_cache(true)
            .global(true)
    }

    pub const fn huge_io_page_flags() -> Self {
        PageEntryFlags::new()
            .present(true)
            .writable(true)
            .huge_page(true)
            .disable_cache(true)
            .global(true)
    }
}
