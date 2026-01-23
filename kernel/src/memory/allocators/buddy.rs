use core::ptr::{self, NonNull};

use common::{
    address_types::PhysicalAddress,
    enums::{BUDDY_MAX_ORDER, BuddyOrder, MemoryRegionType},
};
use cpu_utils::structures::paging::PageTable;

use crate::memory::{
    memory_map::ParsedMemoryMap,
    page::{PAGES, UnassignedPage, meta::BuddyPageMeta},
    unassigned::Unassigned,
};

pub static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator {
    freelist: [const { BuddyPageMeta::default() }; BUDDY_MAX_ORDER],
};

pub struct BuddyAllocator {
    freelist: [BuddyPageMeta; BUDDY_MAX_ORDER],
}

impl BuddyAllocator {
    pub fn alloc_pages(&mut self, num_pages: usize) -> PhysicalAddress {
        assert!(
            num_pages <= (1 << BuddyOrder::MAX as usize),
            "Size cannot be greater then: {}",
            1 << BuddyOrder::MAX as usize
        );
        let order = (usize::BITS
            - 1
            - num_pages.next_power_of_two().leading_zeros())
            as usize;

        let page = self.freelist[order].detach().unwrap_or_else(|| {
            self.split_until(order)
                .expect("Out of memory, swap is not implemented")
        });

        unsafe { page.as_ref().physical_address() }
    }

    // pub fn free_pages(&self, address: usize) {
    //     let page_index = address / REGULAR_PAGE_SIZE;
    // }

    /// This function assumes that `wanted_order` is empty, and won't check
    /// it.
    pub fn split_until(
        &mut self,
        wanted_order: usize,
    ) -> Option<NonNull<UnassignedPage>> {
        let mut closet_order = ((wanted_order + 1)..BUDDY_MAX_ORDER)
            .find(|i| self.freelist[*i].next.is_some())?;

        let initial_page = unsafe {
            self.freelist[closet_order]
                .detach::<Unassigned>()
                .unwrap()
                .as_mut()
        };

        let (mut lhs, mut rhs) = unsafe { initial_page.split() }.unwrap();
        closet_order -= 1;

        while closet_order != wanted_order {
            self.freelist[closet_order].attach(rhs);

            let split_ref = unsafe { lhs.as_mut() };

            (lhs, rhs) = unsafe { split_ref.split().unwrap() };
            closet_order -= 1;
        }

        self.freelist[closet_order].attach(rhs);
        Some(lhs)
    }

    pub fn merge_until_max(&self, page: NonNull<UnassignedPage>) {
        todo!()
    }

    pub fn alloc_table(&mut self) -> &'static mut PageTable {
        unsafe {
            let address = self.alloc_pages(1).translate();
            ptr::write_volatile(
                address.as_mut_ptr::<PageTable>(),
                PageTable::empty(),
            );
            &mut *address.as_mut_ptr::<PageTable>()
        }
    }

    /// The code_end number should be the end address of the code.
    ///
    /// This function will not put in the free list pages that hold
    /// addresses from 0->code_end
    pub fn init(&'static mut self, map: ParsedMemoryMap, code_end: usize) {
        for area in map
            .iter()
            .filter(|a| a.region_type == MemoryRegionType::Usable)
        {
            let mut start = UnassignedPage::index_of_page(
                (area.base_address as usize).into(),
            );
            let end = UnassignedPage::index_of_page(
                ((area.base_address + area.length) as usize).into(),
            );

            let mut prev = None;

            while start < end {
                let largest_order = BuddyOrder::try_from(
                    ((end - start).ilog2().min(BuddyOrder::MAX as u32))
                        as u8,
                )
                .unwrap();

                let curr = unsafe { &mut PAGES[start] };
                let next = unsafe {
                    &mut PAGES[start + ((1 << largest_order as usize) - 1)]
                };
                unsafe {
                    (*curr.meta.buddy).next =
                        Some(NonNull::from_mut(next));
                    (*curr.meta.buddy).prev = prev;
                    (*curr.meta.buddy).order = Some(largest_order);
                }
                prev = Some(NonNull::from_mut(curr));

                self.freelist[largest_order as usize]
                    .attach(NonNull::from_mut(curr));

                start += 1 << largest_order as usize;
            }
        }
    }
}
#[macro_export]
/// Allocate the amount of pages specified, and return the address
macro_rules! alloc_pages {
    ($page_number: expr) => {{
        use $crate::memory::allocators::buddy::BUDDY_ALLOCATOR;
        BUDDY_ALLOCATOR.alloc_pages($page_number)
    }};
}
