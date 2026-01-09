use crate::{
    memory::{allocators::slab::SlabCache, memory_map::ParsedMemoryMap},
    println,
};
use common::{
    constants::{
        PAGE_ALLOCATOR_OFFSET, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
    },
    enums::{BUDDY_MAX_ORDER, BuddyOrder},
    write_volatile,
};
use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

pub static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator {
    freelist: [BuddyBlockMeta {
        next: None,
        prev: None,
        order: None,
    }; BUDDY_MAX_ORDER],
};

#[derive(Default, Debug)]
pub struct Unassigned;

pub type UnassignedPage = Page<Unassigned>;

impl UnassignedPage {
    pub fn assign<T>(&self) -> &Page<T> {
        let ptr = self as *const _ as usize;
        unsafe { &*(ptr as *const Page<T>) }
    }

    pub fn assign_mut<T>(&mut self) -> &mut Page<T> {
        let ptr = self as *const _ as usize;
        unsafe { &mut *(ptr as *mut Page<T>) }
    }
}

pub static mut PAGES: LateInit<&'static mut [UnassignedPage]> =
    LateInit::uninit();

#[derive(Default, Clone, Copy, Debug)]
pub struct BuddyBlockMeta {
    next: Option<*mut UnassignedPage>,
    prev: Option<*mut UnassignedPage>,
    order: Option<BuddyOrder>,
}

impl BuddyBlockMeta {
    pub fn detach<T>(&mut self) -> Option<*mut Page<T>> {
        let detached = self.next? as *mut Page<T>; // None if there is no page to detach
        self.next = unsafe { (*detached).buddy_meta.next };
        Some(detached)
    }

    pub fn attach<T>(&mut self, attachment: *mut Page<T>) {
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

    pub fn free_pages(&self, address: usize) {
        let page_index = address / REGULAR_PAGE_SIZE;
    }

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
    }
}

#[derive(Debug)]
pub struct Page<T: 'static> {
    pub owner: Option<&'static SlabCache<T>>,
    pub buddy_meta: BuddyBlockMeta,
}

impl<T: 'static> Page<T> {
    pub fn as_unassigned(&self) -> &UnassignedPage {
        let ptr = self as *const _ as usize;
        unsafe { &*(ptr as *const UnassignedPage) }
    }

    pub fn as_unassigned_mut(&mut self) -> &mut UnassignedPage {
        let ptr = self as *const _ as usize;
        unsafe { &mut *(ptr as *mut UnassignedPage) }
    }

    pub fn physical_address(&self) -> usize {
        let index = (self.as_unassigned() as *const _ as usize
            - PAGE_ALLOCATOR_OFFSET)
            / size_of::<UnassignedPage>();

        index * REGULAR_PAGE_SIZE
    }

    pub fn get_buddy(&self) -> Option<*mut Page<T>> {
        let order = self.buddy_meta.order?;
        if let BuddyOrder::MAX = order {
            None
        } else {
            Some(
                (self as *const _ as usize
                    ^ ((1 << order as usize)
                        * size_of::<UnassignedPage>()))
                    as *mut Page<T>,
            )
        }
    }

    /// TODO: Make an unsafe split if relevant
    ///
    /// # Safety
    /// This function does not attach the new references!
    pub unsafe fn split(
        &mut self,
    ) -> Option<(*mut Page<T>, *mut Page<T>)> {
        // Reduce it's order to find it's order.

        let prev_order =
            BuddyOrder::try_from(self.buddy_meta.order? as u8 - 1)
                .unwrap();

        write_volatile!(self.buddy_meta.order, Some(prev_order));
        let index = ((self.as_unassigned() as *const _ as usize
            - PAGE_ALLOCATOR_OFFSET)
            / size_of::<UnassignedPage>())
            + (1 << prev_order as usize);

        // Find it's half
        let buddy = unsafe { PAGES[index].assign_mut::<T>() };

        // Set the order of the buddy.
        write_volatile!(buddy.buddy_meta.order, Some(prev_order));

        Some((self as *mut Page<T>, buddy as *mut Page<T>))
    }
}

pub struct LateInit<T>(MaybeUninit<T>);

impl<T> LateInit<T> {
    pub const fn uninit() -> LateInit<T> {
        LateInit::<T>(MaybeUninit::uninit())
    }

    pub const fn write(&mut self, val: T) {
        self.0.write(val);
    }
}

impl<T> Deref for LateInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.assume_init_ref() }
    }
}

impl<T> DerefMut for LateInit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.assume_init_mut() }
    }
}

pub fn pages_init(map: &ParsedMemoryMap) -> usize {
    let last = map.last().unwrap();
    let last_page = (last.base_address + last.length) as usize
        & !REGULAR_PAGE_ALIGNMENT.as_usize();
    let total_pages = last_page / REGULAR_PAGE_SIZE;

    println!(
        "Last Page: {}, Total Pages: {}, size_of_array: {:x?} Kib",
        last_page,
        total_pages,
        total_pages * size_of::<Page<Unassigned>>() / 1024
    );
    unsafe {
        PAGES.write(core::slice::from_raw_parts_mut(
            PAGE_ALLOCATOR_OFFSET as *mut UnassignedPage,
            total_pages,
        ));

        for p in PAGES.iter_mut() {
            core::ptr::write_volatile(
                p as *mut UnassignedPage,
                UnassignedPage {
                    buddy_meta: BuddyBlockMeta {
                        next: None,
                        order: None,
                        prev: None,
                    },
                    owner: None,
                },
            );
        }
        PAGES.as_ptr_range().end as usize
    }
}
