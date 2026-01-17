use core::ptr::{self, NonNull};

use common::{
    address_types::PhysicalAddress,
    constants::REGULAR_PAGE_SIZE,
    enums::{BUDDY_MAX_ORDER, BuddyOrder},
};
use cpu_utils::structures::paging::PageTable;

use crate::{
    memory::{
        allocators::buddy::meta::BuddyBlockMeta,
        page_descriptor::{PAGES, Page, Unassigned, UnassignedPage},
    },
    println,
};

pub mod meta;

pub static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator {
    freelist: [BuddyBlockMeta {
        next: None,
        prev: None,
        order: None,
    }; BUDDY_MAX_ORDER],
};

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
            curr.buddy_meta.next = iter.peek().map(|v| unsafe {
                NonNull::new_unchecked(
                    *v as *const Page<Unassigned> as *mut UnassignedPage,
                )
            });
            curr.buddy_meta.prev = prev;
            curr.buddy_meta.order = Some(BuddyOrder::MAX);
            prev = Some(NonNull::from_mut(curr))
        }
        self.freelist[BUDDY_MAX_ORDER - 1] = BuddyBlockMeta {
            next: Some(unsafe { NonNull::new_unchecked(&mut PAGES[0]) }),
            prev: None,
            order: Some(BuddyOrder::MAX),
        };
        // Allocate initial MB

        // Allocate pages array
        let mem_map_size_pages = unsafe {
            (PAGES.len() * size_of::<UnassignedPage>()) / REGULAR_PAGE_SIZE
        };
        println!("Mem map pages total: {}", mem_map_size_pages);
        println!(
            "Mem Map allocation: {:x?}",
            self.alloc_pages(256 + mem_map_size_pages)
        );
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
