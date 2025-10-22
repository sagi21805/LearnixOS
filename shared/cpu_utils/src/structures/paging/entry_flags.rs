macro_rules! table_entry_flags {
    () => {
        // Is this page present?
        common::flag!(present, 0);

        // Is this page writable?
        common::flag!(writable, 1);

        // Can this page be accessed from user mode
        common::flag!(usr_access, 2);

        // Writes go directly to memory
        common::flag!(write_through_cache, 3);

        // Disable cache for this page
        common::flag!(disable_cache, 4);

        // This flag can help identifying if an entry is the
        // last one, or it is pointing to another directory
        // Is this page points to a custom memory address
        // and not a page table?
        common::flag!(huge_page, 7);

        // Page isn’t flushed from caches on address space
        // switch (PGE bit of CR4 register must be set)
        common::flag!(global, 8);

        // mark a table as full
        common::flag!(full, 9);

        // This entry points to a table
        common::flag!(table, 10);

        // This entry is at the top of the hierarchy.
        common::flag!(root_entry, 11);

        // This page is holding data and is not executable
        common::flag!(not_executable, 63);
    };
}

/// A wrapper for `PageTableEntry` flags for easier use
#[derive(Debug, Clone)]
pub struct PageEntryFlags(u64);

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

    /// Return the underlying flags as u64.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}
