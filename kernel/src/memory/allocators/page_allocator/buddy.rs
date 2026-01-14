use core::ptr;

use common::{
    address_types::PhysicalAddress,
    enums::{BUDDY_MAX_ORDER, BuddyOrder},
};
use cpu_utils::structures::paging::PageTable;

use crate::memory::{
    allocators::slab::SlabPosition,
    page_descriptor::{PAGES, Page, Unassigned, UnassignedPage},
};

pub static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator {
    freelist: [BuddyBlockMeta {
        next: None,
        prev: None,
        order: None,
    }; BUDDY_MAX_ORDER],
};

#[derive(Default, Clone, Copy, Debug)]
pub struct BuddyBlockMeta {
    // TODO CHANGE INTO REF BECAUSE IT CONSUMES LESS MEMORY
    pub next: Option<*mut UnassignedPage>,
    pub prev: Option<*mut UnassignedPage>,
    pub order: Option<BuddyOrder>,
}

impl BuddyBlockMeta {
    pub fn detach<T: SlabPosition>(&mut self) -> Option<*mut Page<T>> {
        let detached = self.next? as *mut Page<T>; // None if there is no page to detach
        self.next = unsafe { (*detached).buddy_meta.next };
        Some(detached)
    }

    pub fn attach<T: SlabPosition>(&mut self, attachment: *mut Page<T>) {
        let attachment_ref =
            unsafe { &mut *attachment }.as_unassigned_mut();
        attachment_ref.buddy_meta.next = self.next;
        self.next = Some(attachment_ref as *mut UnassignedPage)
    }
}

pub struct BuddyAllocator {
    freelist: [BuddyBlockMeta; BUDDY_MAX_ORDER],
}

impl BuddyAllocator {
    pub fn alloc_pages(&mut self, num_pages: usize) -> usize {
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

        (unsafe { &*page }).physical_address()
    }

    // pub fn free_pages(&self, address: usize) {
    //     let page_index = address / REGULAR_PAGE_SIZE;
    // }

    /// This function assumes that `wanted_order` is empty, and won't check
    /// it.
    pub fn split_until(
        &mut self,
        wanted_order: usize,
    ) -> Option<*mut UnassignedPage> {
        let mut closet_order = ((wanted_order + 1)..BUDDY_MAX_ORDER)
            .find(|i| self.freelist[*i].next.is_some())?;

        let initial_page = unsafe {
            &mut *self.freelist[closet_order]
                .detach::<Unassigned>()
                .unwrap()
        };

        let (mut lhs, mut rhs) = unsafe { initial_page.split() }.unwrap();
        closet_order -= 1;

        while closet_order != wanted_order {
            self.freelist[closet_order].attach(rhs);

            let split_ref = unsafe { &mut *lhs };

            (lhs, rhs) = unsafe { split_ref.split().unwrap() };
            closet_order -= 1;
        }

        self.freelist[closet_order].attach(rhs);
        Some(lhs)
    }

    pub fn merge(&self) {
        unimplemented!()
    }

    pub fn alloc_table(&mut self) -> &'static mut PageTable {
        unsafe {
            let address =
                { PhysicalAddress::new_unchecked(self.alloc_pages(1)) };
            ptr::write_volatile(
                address.as_mut_ptr::<PageTable>(),
                PageTable::empty(),
            );
            &mut *address.as_mut_ptr::<PageTable>()
        }
    }

    pub fn init(&'static mut self) {
        let mut iter = unsafe {
            PAGES
                .iter_mut()
                .step_by(1 << BuddyOrder::MAX as usize)
                .peekable()
        };

        let mut prev = None;

        while let Some(curr) = iter.next() {
            curr.buddy_meta.next = iter.peek().map(|v| {
                *v as *const UnassignedPage as *mut UnassignedPage
            });
            curr.buddy_meta.prev = prev;
            curr.buddy_meta.order = Some(BuddyOrder::MAX);
            prev = Some(curr)
        }
        self.freelist[BUDDY_MAX_ORDER - 1] = BuddyBlockMeta {
            next: Some(unsafe { (&mut PAGES[0]) as *mut UnassignedPage }),
            prev: None,
            order: Some(BuddyOrder::MAX),
        };
        // Allocate initial MB
        self.alloc_pages(256);
    }
}
