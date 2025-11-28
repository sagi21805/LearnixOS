use core::{
    alloc::{AllocError, Allocator, Layout},
    cell::UnsafeCell,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use common::{
    address_types::{PhysicalAddress, VirtualAddress},
    bitmap::{BitMap, ContiguousBlockLayout, Position},
    constants::{
        FIRST_STAGE_OFFSET, PAGE_ALLOCATOR_OFFSET, PHYSICAL_MEMORY_OFFSET,
        REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
    },
    enums::MemoryRegionType,
};
use cpu_utils::structures::paging::PageTable;

use crate::parsed_memory_map;

#[derive(Debug)]
// TODO: This is not thread safe, probably should use Mutex
// in the future
/// Physical page allocator implemented with a bitmap, every
/// bit corresponds to a physical page
pub struct PhysicalPageAllocator(UnsafeCell<BitMap>);

impl Clone for PhysicalPageAllocator {
    fn clone(&self) -> Self {
        unsafe {
            let bitmap = self.map_mut();
            Self(UnsafeCell::new(bitmap.clone()))
        }
    }
}

impl PhysicalPageAllocator {
    /// Creates a new allocator from the `bitmap_address`
    /// and the `memory_size`.
    ///
    /// # Parameters
    ///
    /// - `bitmap_address`: Virtual address that is identity mapped and
    ///   will use to store the map
    /// - `memory_size`: Memory size in <u>bytes</u>
    #[allow(unsafe_op_in_unsafe_fn)]
    pub const unsafe fn new(
        bitmap_address: VirtualAddress,
        memory_size: usize,
    ) -> PhysicalPageAllocator {
        let size_in_pages = memory_size / REGULAR_PAGE_SIZE;
        let map_size = size_in_pages / u64::BITS as usize;
        PhysicalPageAllocator(UnsafeCell::new(BitMap::new(
            bitmap_address,
            map_size,
        )))
    }

    pub const fn address_position(
        address: PhysicalAddress,
    ) -> Option<Position> {
        if address.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            let bit_index = address.as_usize() / REGULAR_PAGE_SIZE;
            return Some(Position::from_abs_bit_index(bit_index));
        }
        None
    }

    unsafe fn map(&self) -> &BitMap {
        unsafe { self.0.as_ref_unchecked() }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn map_mut(&self) -> &mut BitMap {
        unsafe { self.0.as_mut_unchecked() }
    }

    pub fn init(uninit: &'static mut MaybeUninit<Self>) {
        unsafe {
            let memory_size = parsed_memory_map!()
                .iter()
                .map(|x| x.length as usize)
                .sum::<usize>();
            uninit.write(Self::new(
                PhysicalAddress::new_unchecked(PAGE_ALLOCATOR_OFFSET)
                    .translate(),
                memory_size,
            ));
            let initialized = uninit.assume_init_mut();

            // Set the null page
            initialized
                .map_mut()
                .set_bit(&Position::new_unchecked(0, 0));

            let start_address = const {
                PhysicalAddress::new_unchecked(FIRST_STAGE_OFFSET as usize)
                    .align_down(REGULAR_PAGE_ALIGNMENT)
            };
            let start_position =
                Self::address_position(start_address).unwrap();
            // Allocate the addresses that are used for the
            // code, and for other variables.
            let end_address = PhysicalAddress::new_unchecked(
                PAGE_ALLOCATOR_OFFSET
                    + core::mem::size_of_val(initialized.map().map),
            )
            .align_up(REGULAR_PAGE_ALIGNMENT);
            let size_bits = ((end_address - start_address)
                / REGULAR_PAGE_SIZE)
                .as_usize();
            let block = ContiguousBlockLayout::from_start_size(
                &start_position,
                size_bits,
            );
            initialized
                .map_mut()
                .set_contiguous_block(&start_position, &block);
            for region in parsed_memory_map!() {
                if region.region_type != MemoryRegionType::Usable {
                    let start_address_aligned =
                        PhysicalAddress::new_unchecked(
                            region.base_address as usize
                                & (u64::MAX
                                    ^ (REGULAR_PAGE_SIZE as u64 - 1))
                                    as usize,
                        );
                    let start_position =
                        Self::address_position(start_address_aligned)
                            .unwrap();
                    let size_bits =
                        region.length as usize / REGULAR_PAGE_SIZE;
                    let block = ContiguousBlockLayout::from_start_size(
                        &start_position,
                        size_bits,
                    );
                    initialized
                        .map_mut()
                        .set_contiguous_block(&start_position, &block);
                }
            }
        };
    }

    /// Resolves `map_index` and `bit_index` into actual
    /// physical address
    pub fn resolve_position(p: &Position) -> PhysicalAddress {
        unsafe {
            PhysicalAddress::new_unchecked(
                ((p.map_index * (u64::BITS as usize)) + p.bit_index)
                    * REGULAR_PAGE_SIZE,
            )
        }
    }

    pub fn resolve_address(address: PhysicalAddress) -> Position {
        let starting_bit_idx = address.as_usize() / REGULAR_PAGE_SIZE;
        Position::from_abs_bit_index(starting_bit_idx)
    }

    pub fn available_memory(&self) -> usize {
        unsafe { self.map().count_zeros() * REGULAR_PAGE_SIZE }
    }

    /// Return the physical address of this table
    pub(super) fn alloc_table(&self) -> &'static mut PageTable {
        let free_block = unsafe { self.map().find_free_block(1) };

        match free_block {
            Some((p, _)) => unsafe {
                let physical_address = Self::resolve_position(&p);

                ptr::write(
                    physical_address.translate().as_mut_ptr::<PageTable>(),
                    PageTable::empty(),
                );

                self.map_mut().set_bit(&p);

                &mut *physical_address.as_mut_ptr::<PageTable>()
            },

            None => panic!(
                "No physical memory is available to allocate this table"
            ),
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl Allocator for PhysicalPageAllocator {
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            match layout.align_to(REGULAR_PAGE_ALIGNMENT.as_usize()) {
                Ok(layout) => {
                    match self
                        .map()
                        .find_free_block(layout.size() / REGULAR_PAGE_SIZE)
                    {
                        Some((p, block)) => {
                            self.map_mut()
                                .set_contiguous_block(&p, &block);
                            Ok(NonNull::slice_from_raw_parts(
                                NonNull::new_unchecked(
                                    Self::resolve_position(&p)
                                        .translate()
                                        .as_mut_ptr::<u8>(),
                                ),
                                layout.size(),
                            ))
                        }
                        None => Err(AllocError),
                    }
                }
                Err(_) => Err(AllocError),
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if let Ok(layout) =
            layout.align_to(REGULAR_PAGE_ALIGNMENT.as_usize())
        {
            let start_position =
                Self::resolve_address(PhysicalAddress::new_unchecked(
                    ptr.as_ptr() as usize - PHYSICAL_MEMORY_OFFSET,
                ));
            let block = ContiguousBlockLayout::from_start_size(
                &start_position,
                layout.size() / REGULAR_PAGE_SIZE,
            );
            self.map_mut()
                .unset_contiguous_block(&start_position, &block);
        }
    }
}

unsafe impl Sync for PhysicalPageAllocator {}
