extern crate alloc;

use core::{
    alloc::Layout,
    ptr::{NonNull, null, null_mut},
};

use alloc::alloc::{AllocError, Allocator, GlobalAlloc, alloc};

use bump::BumpAllocator;
use common::{
    address_types::{PhysicalAddress, VirtualAddress},
    enums::{PageSize, PageTableLevel},
    error::{EntryError, MappingError},
    late_init::LateInit,
};
use x86::structures::paging::{PageEntryFlags, PageTable, PageTableEntry};

use crate::{fmt::kprint, println};

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

    pub fn swap(&mut self, allocator: &'a dyn GlobalAlloc) {
        todo!(
            "Ensure allocation is passsed to the next allocator, or it \
             deallocated all"
        );
        self.allocator = LateInit::new(allocator);
    }
}

unsafe impl<'a> GlobalAlloc for GlobalAllocator<'a> {
    unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
        unsafe { kprint(format_args!("Layout: {:?}\n", layout)) };
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

pub fn alloc_table() -> Result<NonNull<PageTable>, AllocError> {
    let ptr = unsafe { alloc(Layout::new::<PageTable>()) };

    Ok(NonNull::new(ptr).ok_or(AllocError)?.cast())
}

#[extend::ext]
pub impl VirtualAddress {
    #[cfg(target_arch = "x86_64")]
    fn walk(&self) -> impl Iterator<Item = NonNull<PageTableEntry>> {
        let mut table = Some(PageTable::current_table());
        let mut level = Some(PageTableLevel::PML4);
        ::core::iter::from_fn(move || {
            let entry =
                unsafe { &table?.as_ref().entries[self.index_of(level?)] };

            if entry.get_flags().is_present() {
                table = entry.mapped_table().ok();
                level = level?.next();
                Some(NonNull::from_ref(entry))
            } else {
                None
            }
        })
    }

    #[cfg(target_arch = "x86_64")]
    fn walk_map(&self) -> impl Iterator<Item = NonNull<PageTableEntry>> {
        let mut table = PageTable::current_table();
        let mut level = Some(PageTableLevel::PML4);
        ::core::iter::from_fn(move || {
            unsafe { kprint(format_args!("Here")) };
            let entry =
                unsafe { &table.as_ref().entries[self.index_of(level?)] };

            unsafe { kprint(format_args!("Table: {:?}\n", table)) };
            table = match entry.mapped_table() {
                Ok(t) => t,
                Err(EntryError::NoMapping) => {
                    alloc_table().expect("Cannot allocate table")
                }
                Err(EntryError::NotATable) => {
                    todo!("Mapping to this entry already exists")
                }
            };
            level = level?.next();
            Some(NonNull::from_ref(entry))
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
            );
        }
        Ok(())
    }
}
