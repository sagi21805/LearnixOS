#![no_std]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![allow(incomplete_features)]
#![feature(explicit_tail_calls)]
#![feature(min_specialization)]
#![feature(ptr_alignment_type)]

pub mod meta;

use core::{alloc::GlobalAlloc, marker::PhantomData, ptr::NonNull};

use libk::println;
use sync::mutex::SpinMutex;
use x86::{memory_map::MemoryMap, structures::paging::VirtualAddressExt};

use common::{
    address_types::{Address, PhysicalAddress},
    alloc::BumpAllocations,
    constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE},
    enums::{BuddyOrder, MemoryRegionType},
    iter,
    volatile::Volatile,
};

use crate::meta::{
    BuddyArena, BuddyBlock, BuddyError, BuddyMeta, Head, Regular,
};

pub struct BuddyAllocator<Arena, Block>
where
    Block: BuddyBlock,
    Arena: BuddyArena<Block>,
{
    arena: SpinMutex<Arena>,
    freelist: SpinMutex<[BuddyMeta<Head>; BuddyOrder::MAX as usize + 1]>,
    // Wrap in a mutex to automatically implement Sync and Send.
    _block: PhantomData<SpinMutex<Block>>,
}

impl<Arena, Block> BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: const BuddyBlock,
{
    pub fn new(memory_map: &MemoryMap) -> BuddyAllocator<Arena, Block> {
        let freelist = SpinMutex::new(
            [BuddyMeta::<Head>::default(); BuddyOrder::MAX as usize + 1],
        );

        let mut lock = freelist.lock();

        let head = &mut lock[0];

        let arena = SpinMutex::new(Arena::new(memory_map, head));

        drop(lock);

        BuddyAllocator {
            arena,
            freelist,
            _block: PhantomData,
        }
    }

    pub fn initialize(
        &self,
        allocations: &BumpAllocations,
        mmap: &MemoryMap,
    ) {
        let len = self.arena.lock().iter().len();

        let mut first = self.arena.lock().at(0).unwrap();

        let lock = self.freelist.lock();
        let head = &lock[0];
        unsafe {
            first.as_mut().meta_mut().prev =
                Volatile::new(NonNull::from_ref(head));
        }

        drop(lock);

        // Allocate all the non-usable regions in the memory map
        if let Some(last_usable_idx) = mmap
            .regions
            .read()
            .iter()
            .rposition(|r| r.region_type == MemoryRegionType::Usable)
        {
            for region in mmap
                .regions
                .read()
                .iter()
                .take(last_usable_idx + 1)
                .filter(|r| r.region_type != MemoryRegionType::Usable)
            {
                let mut block = self
                    .arena
                    .lock()
                    .page_with_address(unsafe {
                        PhysicalAddress::new_unchecked(
                            region.base_address as usize,
                        )
                        .align_down(REGULAR_PAGE_ALIGNMENT)
                    })
                    .unwrap();
                println!("First Block: {:?}", block);

                let mut length = (region.length as usize)
                    .next_multiple_of(REGULAR_PAGE_SIZE);

                while length > 0 {
                    length -= REGULAR_PAGE_SIZE;
                    unsafe {
                        self.allocate_block(block);
                    }
                    block = unsafe { block.add(1) };
                }
                println!(
                    "Allocated Range {:x}..{:x}",
                    PhysicalAddress::new_unchecked(
                        region.base_address as usize,
                    )
                    .align_down(REGULAR_PAGE_ALIGNMENT)
                    .as_usize(),
                    region.base_address + region.length
                )
            }
        }

        // Note: Because this is a hobby OS, I do not hanlde all cases on
        // the ranges in the memory map. And I rely on the observation that
        // the first first matching free range in the arena is a power of
        // 2, which is good for this allocator.
        if allocations.index > 1 {
            todo!(
                "Currently does not implement passing allocations that \
                 are not the arena of this allocator"
            )
        }

        let allocation = allocations.iter().nth(0).unwrap();

        println!("Allocation Base: {:x?}", allocation.base.as_usize());

        let allocation_address = allocation.base.translate().unwrap();

        println!(
            "Initial Allocation Address: {}",
            allocation_address.as_usize()
        );

        let mut block = self
            .arena
            .lock()
            .page_with_address(allocation_address)
            .unwrap();

        println!("Block: {:?}", block);

        let mut length =
            allocation.layout.size().next_multiple_of(REGULAR_PAGE_SIZE);

        while length > 0 {
            length -= REGULAR_PAGE_SIZE;
            unsafe {
                self.allocate_block(block);
            }
            block = unsafe { block.add(1) };
        }

        for (n, _power) in
            iter::power_chunk_firsts(0..len, BuddyOrder::MAX as usize)
        {
            let page = self.arena.lock().at(n).unwrap();
            self.merge_recursive(page);
        }
    }

    /// Mark a block as allocated, detaching it from the freelist.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `block` is a valid free block in the
    /// arena.
    unsafe fn allocate_meta(
        &self,
        mut block: NonNull<BuddyMeta<Regular>>,
    ) {
        debug_assert!(
            unsafe { !block.as_ref().flags.is_allocated() },
            "{:?}",
            block
        );

        // Detach page from the freelist and mark it as allocated
        unsafe {
            block.as_mut().detach();
            block.as_mut().flags.set_allocated(true);
        };
    }

    unsafe fn allocate_block(&self, block: NonNull<Block>) {
        unsafe {
            self.allocate_meta(NonNull::from_ref(block.as_ref().meta()));
        }
    }

    pub fn split_until(
        &self,
        wanted_order: usize,
    ) -> Option<NonNull<Block>> {
        let (closet_order, initial_page) = ((wanted_order + 1)
            ..=BuddyOrder::MAX as usize)
            .find_map(|i| {
                Some((
                    i,
                    Block::from_meta(self.freelist.lock()[i].next.read()?),
                ))
            })?;

        Some(self.split_recursive(
            initial_page,
            closet_order,
            wanted_order,
        ))
    }

    fn split_recursive(
        &self,
        page: NonNull<Block>,
        current_order: usize,
        target_order: usize,
    ) -> NonNull<Block> {
        debug_assert!(target_order < current_order);

        if current_order == target_order {
            return page;
        }

        let (lhs, rhs) = match self.arena.lock().split(page) {
            Ok((lhs, rhs)) => (lhs, rhs),
            Err(e) => todo!("Handle Error, {:?}", e),
        };

        let next_order = current_order - 1;

        self.freelist.lock()[next_order].attach_block(rhs);

        become self.split_recursive(lhs, next_order, target_order)
    }

    /// This function will try to merge a page with it's buddy, until it
    /// cannot be merged anymore.
    pub fn merge_recursive(&self, mut page: NonNull<Block>) -> BuddyOrder {
        let arena = self.arena.lock();
        loop {
            let buddy = match arena.buddy_of(page) {
                Ok(buddy) => buddy,
                Err(
                    BuddyError::BuddyOutOfRange | BuddyError::MaxOrder,
                ) => {
                    let order =
                        unsafe { page.as_ref().meta().flags.get_order() };
                    return order;
                }
                Err(e) => {
                    unreachable!(
                        "Problem in algorithm, the error should not \
                         happen {:?}\n {:?}",
                        e, page,
                    )
                }
            };

            if unsafe { buddy.as_ref().meta().flags.is_allocated() } {
                let order =
                    unsafe { page.as_ref().meta().flags.get_order() };
                return order;
            }

            let buddy_order =
                unsafe { buddy.as_ref().meta().flags.get_order() };
            let page_order =
                unsafe { page.as_ref().meta().flags.get_order() };

            debug_assert!(buddy_order <= page_order);

            if buddy_order < page_order {
                page = buddy;
                continue;
            }
            match arena.merge(page, buddy) {
                Ok(merged) => {
                    self.attach_block(merged);
                    page = merged;
                }
                Err(e) => unreachable!("{}", e),
            };
        }
    }

    fn attach_block(&self, block: NonNull<Block>) {
        let order = unsafe { block.as_ref().meta().flags.get_order() };
        self.freelist.lock()[order as usize].attach_block(block);
    }

    pub fn print_allocated_regions(&self) {
        let arena = self.arena.lock();

        let mut prev_address =
            unsafe { PhysicalAddress::new_unchecked(0) };

        for block in arena.iter() {
            let meta = unsafe { block.as_ref().meta() };

            if meta.flags.is_allocated() {
                let address = arena.address_of(block);
                if address.as_usize() - prev_address.as_usize()
                    > REGULAR_PAGE_SIZE
                {
                    println!("");
                }
                println!("{:x?}", address);
                prev_address = address;
            }
        }
    }
}

impl<Arena, Block> ::core::fmt::Debug for BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: const BuddyBlock,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BuddyAllocator")
            .field("freelist", &self.freelist.try_lock())
            .finish()
    }
}

unsafe impl<Arena, Block> GlobalAlloc for BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: const BuddyBlock,
{
    #[track_caller]
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let num_pages = layout.size().next_multiple_of(
            REGULAR_PAGE_ALIGNMENT.max(layout.alignment()).as_usize(),
        ) / REGULAR_PAGE_SIZE;

        debug_assert!(num_pages <= (1 << BuddyOrder::MAX as usize));

        if num_pages > (1 << BuddyOrder::MAX as usize) || num_pages == 0 {
            return core::ptr::null_mut();
        }

        let order = (usize::BITS
            - 1
            - num_pages.next_power_of_two().leading_zeros())
            as usize;

        let page =
            self.freelist.lock()[order].next.read().unwrap_or_else(|| {
                NonNull::from_ref(unsafe {
                    self.split_until(order)
                        .expect("Out of memory, swap is not implemented")
                        .as_ref()
                        .meta()
                })
            });

        unsafe { self.allocate_meta(page) };

        self.arena
            .lock()
            .address_of(Block::from_meta(page))
            .as_non_null::<u8>()
            .as_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let num_pages = layout.size().next_multiple_of(
            REGULAR_PAGE_ALIGNMENT.max(layout.alignment()).as_usize(),
        );

        debug_assert!(
            num_pages <= (1 << BuddyOrder::MAX as usize),
            "Size cannot be greater then: {}",
            1 << BuddyOrder::MAX as usize
        );
        if num_pages > (1 << BuddyOrder::MAX as usize) || num_pages == 0 {
            panic!(
                "Tried to deallocate a layout that couldn't possibly be \
                 allocated by the allocate function."
            )
        }
        let mut page = self
            .arena
            .lock()
            .page_with_address(PhysicalAddress::from(ptr as usize))
            .unwrap();

        let order = (usize::BITS
            - 1
            - num_pages.next_power_of_two().leading_zeros())
            as usize;

        unsafe {
            page.as_mut()
                .meta_mut()
                .flags
                .set_order(BuddyOrder::from(order as u8));
        }

        // Deallocate the page by attaching it back to the freelist.
        self.freelist.lock()[order].attach_block(page);
    }
}
