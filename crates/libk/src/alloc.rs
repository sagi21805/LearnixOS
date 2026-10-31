extern crate alloc;

use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::{AllocError, Allocator, StaticAllocator, alloc};

use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    enums::{PageSize, PageTableLevel},
    error::{EntryError, MappingError},
};
use sync::once::Once;
use x86::structures::paging::{PageEntryFlags, PageTable, PageTableEntry};

use crate::println;

/// The allocator behind [`GlobalAllocator`].
pub type DynAllocator = dyn Allocator + Send + Sync;

pub struct GlobalAllocator<'a> {
    allocator: Once<&'a DynAllocator>,
}

impl<'a> GlobalAllocator<'a> {
    /// Routes every allocation and free to `allocator`.
    ///
    /// # Panics
    ///
    /// When an allocator was already set.
    pub fn set(&self, allocator: &'a DynAllocator) {
        self.allocator.init(allocator);
    }

    pub const fn uninit() -> Self {
        Self {
            allocator: Once::uninit(),
        }
    }
}

// SAFETY: each method forwards to the one allocator `set` installs, so a
// block stays valid until it is freed.
unsafe impl Allocator for GlobalAllocator<'_> {
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.allocator.allocate(layout)
    }

    fn allocate_zeroed(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.allocator.allocate_zeroed(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: the caller's guarantees carry over.
        unsafe { self.allocator.deallocate(ptr, layout) }
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: the caller's guarantees carry over.
        unsafe { self.allocator.grow(ptr, old_layout, new_layout) }
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: the caller's guarantees carry over.
        unsafe { self.allocator.grow_zeroed(ptr, old_layout, new_layout) }
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: the caller's guarantees carry over.
        unsafe { self.allocator.shrink(ptr, old_layout, new_layout) }
    }
}

// SAFETY: blocks are invalidated only by `deallocate`. Dropping the
// wrapper frees nothing, and the allocator is set once, so it never
// changes under a live block.
unsafe impl StaticAllocator for GlobalAllocator<'static> {}

// SAFETY: the buddy allocator behind it never allocates while
// allocating or freeing, so a call never recurses into the global
// allocator.
unsafe impl core::alloc::GlobalAllocator for GlobalAllocator<'static> {}

/// Allocate a new table, point to it from a given [`PageTableEntry`]
pub fn alloc_table(
    entry: &mut PageTableEntry,
) -> Result<NonNull<PageTable>, AllocError> {
    let ptr = unsafe { alloc(Layout::new::<PageTable>()) };
    let table = NonNull::new(ptr).ok_or(AllocError)?.cast::<PageTable>();
    unsafe {
        entry.map(
            PhysicalAddress::new_unchecked(ptr as usize),
            PageEntryFlags::table_flags(),
        );
    }
    Ok(table)
}

#[extend::ext(name = VirtualAddressMapping)]
pub impl VirtualAddress {
    #[cfg(target_arch = "x86_64")]
    fn walk_map(&self) -> impl Iterator<Item = NonNull<PageTableEntry>> {
        let mut table = PageTable::current_table();
        let mut level = Some(PageTableLevel::PML4);
        let mut prev_entry: Option<NonNull<PageTableEntry>> = None;
        ::core::iter::from_fn(move || {
            if let Some(mut prev_entry) = prev_entry {
                table = match unsafe { prev_entry.as_ref().mapped_table() }
                {
                    Ok(t) => t,
                    Err(EntryError::NoMapping) => {
                        println!("ALLOCATED NEW TABLE");
                        alloc_table(unsafe { prev_entry.as_mut() })
                            .expect("Cannot allocate table")
                    }
                    Err(EntryError::NotATable) => {
                        todo!("Mapping to this entry already exists")
                    }
                };
            }

            let current_level = level?;
            let entry = unsafe {
                NonNull::from_ref(
                    &table.as_mut().entries[self.index_of(current_level)],
                )
            };
            level = current_level.next();
            prev_entry = Some(entry);

            Some(entry)
        })
    }

    #[cfg(target_arch = "x86_64")]
    fn map(
        &self,
        address: PhysicalAddress,
        flags: Option<PageEntryFlags>,
        page_size: PageSize,
    ) -> Result<(), MappingError> {
        let mut entry = self
            .walk_map()
            .nth(page_size.mapping_table() as usize)
            .ok_or(MappingError::TableDoesNotExist)?;

        unsafe {
            entry.as_mut().map(
                address,
                flags.unwrap_or_else(|| match page_size {
                    PageSize::Regular => {
                        PageEntryFlags::regular_page_flags()
                    }
                    PageSize::Big | PageSize::Huge => {
                        PageEntryFlags::huge_page_flags()
                    }
                }),
            )
        }
        Ok(())
    }
}
