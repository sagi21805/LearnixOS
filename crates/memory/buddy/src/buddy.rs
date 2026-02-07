use core::ptr::{self, NonNull};

use common::{
    address_types::PhysicalAddress,
    enums::{BUDDY_MAX_ORDER, BuddyOrder, MemoryRegionType},
    write_volatile,
};
use cpu_utils::structures::paging::PageTable;

use crate::memory::{
    memory_map::ParsedMemoryMap,
    page::{PAGES, UnassignedPage, meta::BuddyPageMeta},
};

pub static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator {
    freelist: [const { BuddyPageMeta::default() }; BUDDY_MAX_ORDER],
};

#[macro_export]
/// Allocate the amount of pages specified, and return the address
macro_rules! alloc_pages {
    ($page_number: expr) => {{
        use $crate::memory::allocators::buddy::BUDDY_ALLOCATOR;
        BUDDY_ALLOCATOR.alloc_pages($page_number)
    }};
}

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
        let closet_order = ((wanted_order + 1)..BUDDY_MAX_ORDER)
            .find(|i| self.freelist[*i].next.is_some())?;

        let initial_page =
            self.freelist[closet_order].detach::<()>().unwrap();

        Some(self.split_recursive(
            initial_page,
            closet_order,
            wanted_order,
        ))
    }

    fn split_recursive(
        &mut self,
        page: NonNull<UnassignedPage>,
        current_order: usize,
        target_order: usize,
    ) -> NonNull<UnassignedPage> {
        debug_assert!(
            target_order < current_order,
            "Target order cannot be greater then current order"
        );

        if current_order == target_order {
            return page;
        }

        let (lhs, rhs) = unsafe { BuddyAllocator::split(page).unwrap() };

        let next_order = current_order - 1;
        self.freelist[next_order].attach(rhs);

        become self.split_recursive(lhs, next_order, target_order)
    }

    /// This function will try to merge a page on the buddy allocator until
    pub fn merge_recursive(&self, page: NonNull<UnassignedPage>) {
        if let Some(merged) =
            unsafe { BuddyAllocator::merge_with_buddy(page) }
        {
            become BuddyAllocator::merge_recursive(self, merged);
        }
    }

    pub fn alloc_table(&mut self) -> NonNull<PageTable> {
        unsafe {
            let address = self.alloc_pages(1).translate();
            ptr::write_volatile(
                address.as_non_null::<PageTable>().as_ptr(),
                PageTable::empty(),
            );
            address.as_non_null::<PageTable>()
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
            let mut start = UnassignedPage::index_of(
                (area.base_address as usize).into(),
            );
            let end = UnassignedPage::index_of(
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

impl BuddyAllocator {
    /// TODO: Make an unsafe split if relevant
    ///
    /// # Safety
    /// This function does not attach the new references!
    #[allow(clippy::type_complexity)]
    unsafe fn split(
        mut page: NonNull<UnassignedPage>,
    ) -> Option<(NonNull<UnassignedPage>, NonNull<UnassignedPage>)> {
        // Reduce it's order to find it's order.
        let prev_order = BuddyOrder::try_from(
            unsafe { page.as_ref().meta.buddy.order? } as u8 - 1,
        )
        .expect("Page order cannot be reduced");

        write_volatile!(
            (*page.as_mut().meta.buddy).order,
            Some(prev_order)
        );

        let index = unsafe {
            ((page.as_ref() as *const _ as usize - PAGES.as_ptr().addr())
                / size_of::<UnassignedPage>())
                + (1 << prev_order as usize)
        };

        // Find it's half
        let mut buddy = unsafe { NonNull::from_mut(&mut PAGES[index]) };

        // Set the order of the buddy.
        write_volatile!(
            (*buddy.as_mut().meta.buddy).order,
            Some(prev_order)
        );

        Some((page, buddy))
    }

    /// This function will detach the given page and it's buddy from their
    /// freelist, increase their and attach to the increased order
    /// list.
    unsafe fn merge_with_buddy(
        page: NonNull<UnassignedPage>,
    ) -> Option<NonNull<UnassignedPage>> {
        let buddy = BuddyAllocator::buddy_of(page)?;

        let next_order = BuddyOrder::try_from(unsafe {
            page.as_ref().meta.buddy.order.unwrap() as u8 + 1
        })
        .unwrap();

        BuddyAllocator::detach_from_mid(page);
        BuddyAllocator::detach_from_mid(buddy);

        // Operate on the page that it's address is lower.
        let (mut left, mut right) = if page < buddy {
            (page, buddy)
        } else {
            (buddy, page)
        };

        unsafe {
            (*left.as_mut().meta.buddy).order = Some(next_order);
            (*right.as_mut().meta.buddy) = BuddyPageMeta::default();
        };

        Some(left)
    }

    // TODO: This function will probably fail, should change that the head
    // of the page list is static and the list starts from the second
    // node, and then this would work
    fn detach_from_mid(page: NonNull<UnassignedPage>) {
        let (mut prev, next) = unsafe {
            let p_ref = page.as_ref();
            (
                p_ref.meta.buddy.prev.expect("Page has no prev"),
                p_ref.meta.buddy.next.expect("Page has no next"),
            )
        };

        unsafe { (*prev.as_mut().meta.buddy).next = Some(next) }
    }

    fn buddy_of(
        page: NonNull<UnassignedPage>,
    ) -> Option<NonNull<UnassignedPage>> {
        let order = unsafe { page.as_ref().meta.buddy.order? };
        if let BuddyOrder::MAX = order {
            None
        } else {
            unsafe {
                let buddy_address = page.as_ref() as *const _ as usize
                    ^ ((1 << order as usize)
                        * size_of::<UnassignedPage>());

                Some(NonNull::new_unchecked(
                    buddy_address as *mut UnassignedPage,
                ))
            }
        }
    }
}
