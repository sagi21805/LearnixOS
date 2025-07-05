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

        // This flag can help identifying if an entry is the last one, or it is pointing to another directory
        // Is this page points to a custom memory address and not a page table?
        common::flag!(huge_page, 7);

        // Page isn’t flushed from caches on address space switch (PGE bit of CR4 register must be set)
        common::flag!(global, 8);

        // mark a table as full
        common::flag!(full, 9);

        // This entry points to a table
        common::flag!(table, 10);

        // This entry is at the top of the heirarchy.
        common::flag!(root_entry, 11);

        // This page is holding data and is not executable
        common::flag!(not_executable, 63);
    };
}

// Just a wrapper for the flags for easier use
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PageEntryFlags(u64);

impl PageEntryFlags {
    table_entry_flags!();

    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn table_flags() -> Self {
        PageEntryFlags::new().present().writable().table()
    }

    pub const fn huge_page_flags() -> Self {
        PageEntryFlags::new().present().writable().huge_page()
    }

    pub const fn regular_page_flags() -> Self {
        PageEntryFlags::new().present().writable()
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}
