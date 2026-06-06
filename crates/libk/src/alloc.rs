extern crate alloc;

use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::{AllocError, GlobalAlloc, alloc};

use bump::BumpAllocator;
use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    enums::{PageSize, PageTableLevel},
    error::{EntryError, MappingError},
    late_init::LateInit,
};
use x86::structures::paging::{PageEntryFlags, PageTable, PageTableEntry};

pub static mut BUMP_ALLOCATOR: LateInit<BumpAllocator> =
    LateInit::uninit();

pub struct GlobalAllocator<'a> {
    allocator: LateInit<&'a dyn GlobalAlloc>,
}

impl<'a> GlobalAllocator<'a> {
    pub fn init(&mut self, allocator: &'a dyn GlobalAlloc) {
        self.allocator = LateInit::new(allocator);
    }

    pub const fn uninit() -> Self {
        Self {
            allocator: LateInit::uninit(),
        }
    }

    pub fn swap(&mut self, _allocator: &'a dyn GlobalAlloc) {
        todo!(
            "Ensure allocation is passsed to the next allocator, or it \
             deallocated all"
        );
        // self.allocator = LateInit::new(allocator);
    }
}

unsafe impl<'a> GlobalAlloc for GlobalAllocator<'a> {
    unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
        unsafe { self.allocator.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::alloc::Layout) {
        unsafe { self.allocator.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { self.allocator.alloc_zeroed(layout) }
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        unsafe { self.allocator.realloc(ptr, layout, new_size) }
    }
}

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

#[extend::ext]
pub impl VirtualAddress {
    #[cfg(target_arch = "x86_64")]
    fn walk(&self) -> impl Iterator<Item = NonNull<PageTableEntry>> {
        let mut table = Some(PageTable::current_table());
        let mut level = Some(PageTableLevel::PML4);
        ::core::iter::from_fn(move || {
            let current_level = level?;

            let entry = unsafe {
                &table?.as_ref().entries[self.index_of(current_level)]
            };

            if entry.get_flags().is_present() {
                table = entry.mapped_table().ok();
                level = current_level.next();
            } else {
                // Stop at the next iteration.
                level = None;
            }
            Some(NonNull::from_ref(entry))
        })
    }

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

    fn is_mapped(&self) -> bool {
        self.walk()
            .all(|e| unsafe { e.as_ref().get_flags().is_present() })
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
