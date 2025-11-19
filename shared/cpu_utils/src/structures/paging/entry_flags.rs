// ANCHOR: table_entry_flags
macro_rules! table_entry_flags {
    () => {
        // Is this page present?
        learnix_macros::flag!(present, 0);

        // Is this page writable?
        learnix_macros::flag!(writable, 1);

        // Can this page be accessed from user mode
        learnix_macros::flag!(usr_access, 2);

        // Writes go directly to memory
        learnix_macros::flag!(write_through_cache, 3);

        // Disable cache for this page
        learnix_macros::flag!(disable_cache, 4);

        // This flag can help identifying if an entry is the
        // last one, or it is pointing to another directory
        // Is this page points to a custom memory address
        // and not a page table?
        learnix_macros::flag!(huge_page, 7);

        // Page isn't flushed from caches on address space
        // switch (PGE bit of CR4 register must be set)
        learnix_macros::flag!(global, 8);

        // 9-11 are custom custom flags for our use
        // mark a table as full
        learnix_macros::flag!(full, 9);

        // This entry points to a table
        learnix_macros::flag!(table, 10);

        // This entry is at the top of the hierarchy.
        learnix_macros::flag!(root_entry, 11);

        // This page is holding data and is not executable
        learnix_macros::flag!(not_executable, 63);
    };
}
// ANCHOR_END: table_entry_flags

// ANCHOR: page_entry_flags
/// A wrapper for `PageTableEntry` flags for easier use
#[derive(Debug, Clone)]
pub struct PageEntryFlags(pub u64);
// ANCHOR_END: page_entry_flags

// ANCHOR: impl_page_entry_flags
impl PageEntryFlags {
    table_entry_flags!();

    /// Constructs new flags, with all flags turned off.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Default flags for entry that contains page table.
    pub const fn table_flags() -> Self {
        PageEntryFlags::new().present().writable().table()
    }

    /// Default flags for entry that contains huge page.
    pub const fn huge_page_flags() -> Self {
        PageEntryFlags::new().present().writable().huge_page()
    }

    /// Default flags for entry that contains regular page.
    pub const fn regular_page_flags() -> Self {
        PageEntryFlags::new().present().writable()
    }
}
// ANCHOR_END: impl_page_entry_flags
