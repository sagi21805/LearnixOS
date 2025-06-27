use crate::memory::bitmap::{BitMap, ContiguousBlockLayout, Position};
use crate::{parsed_memory_map, println};
use common::constants::addresses::FIRST_STAGE_OFFSET;
use common::constants::enums::MemoryRegionType;
use common::constants::values::REGULAR_PAGE_ALIGNMENT;
use common::constants::{addresses::PAGE_ALLOCATOR_OFFSET, values::REGULAR_PAGE_SIZE};
use core::mem::MaybeUninit;
use core::ptr::{self, Alignment, null};
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
};
use cpu_utils::structures::paging::address_types::{PhysicalAddress, VirtualAddress};
use cpu_utils::structures::paging::page_tables::PageTable;
#[derive(Debug)]
// TODO: This is not thread safe, probably should use Mutex in the future
/// Physical page allocator implemented with a bitmap, every bit corresponds to a physical page
pub struct PhysicalPageAllocator(UnsafeCell<BitMap>);

impl PhysicalPageAllocator {
    /// Constructs a new physical page allocator using the provided bitmap address and total memory size.
    ///
    /// The allocator uses the given virtual address to store its internal bitmap, which tracks allocation status for each physical page. The memory size determines the number of pages managed by the allocator. The bitmap address must be identity mapped.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided bitmap address is valid, properly aligned, and mapped for the required size.
    ///
    /// # Examples
    ///
    /// ```
    /// let allocator = unsafe { PhysicalPageAllocator::new(bitmap_addr, total_memory_bytes) };
    /// ```
    #[allow(unsafe_op_in_unsafe_fn)]
    pub const unsafe fn new(
        bitmap_address: VirtualAddress,
        memory_size: usize,
    ) -> PhysicalPageAllocator {
        let size_in_pages = memory_size / (REGULAR_PAGE_SIZE * u64::BITS as usize);
        PhysicalPageAllocator(UnsafeCell::new(BitMap::new(bitmap_address, size_in_pages)))
    }

    /// Returns the bitmap position corresponding to a physical address if it is page-aligned.
    ///
    /// Returns `None` if the address is not aligned to the regular page size. Otherwise, computes the bit index representing the page and returns its position in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// let addr = PhysicalAddress::from(0x2000);
    /// let pos = PhysicalPageAllocator::address_position(addr);
    /// assert!(pos.is_some());
    /// ```
    pub const fn address_position(address: PhysicalAddress) -> Option<Position> {
        if address.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            let bit_index = address.as_usize() / REGULAR_PAGE_SIZE;
            return Some(Position::from_abs_bit_index(bit_index));
        }
        None
    }

    /// Returns an immutable reference to the internal bitmap representing physical page allocation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no mutable references to the bitmap exist while this reference is in use, as the allocator is not thread-safe.
    ///
    /// # Examples
    ///
    /// ```
    /// // SAFETY: Caller must ensure exclusive access if mutating elsewhere.
    /// let bitmap = unsafe { allocator.map() };
    /// ```
    unsafe fn map(&self) -> &BitMap {
        unsafe { self.0.as_ref_unchecked() }
    }

    /// Returns a mutable reference to the internal bitmap used for tracking physical page allocation.
    ///
    /// # Safety
    ///
    /// The caller must ensure exclusive access to the allocator to prevent data races, as this method provides mutable access to the underlying bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// // SAFETY: Caller must ensure exclusive access to the allocator.
    /// let bitmap = unsafe { allocator.map_mut() };
    /// ```
    unsafe fn map_mut(&self) -> &mut BitMap {
        unsafe { self.0.as_mut_unchecked() }
    }

    /// Initializes the physical page allocator in-place using the parsed memory map.
    ///
    /// This function sets up the allocator at a fixed physical address, marks pages used by the kernel and allocator metadata as allocated, and reserves all non-usable memory regions in the bitmap. It must be called before any allocations are performed.
    ///
    /// # Parameters
    /// - `uninit`: A mutable reference to uninitialized memory where the allocator will be constructed.
    ///
    /// # Safety
    /// This function performs raw pointer operations and assumes the provided memory is valid for initialization.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut uninit = core::mem::MaybeUninit::uninit();
    /// PhysicalPageAllocator::init(&mut uninit);
    /// let allocator = unsafe { uninit.assume_init() };
    /// ```
    pub fn init(uninit: &mut MaybeUninit<Self>) {
        unsafe {
            let memory_size = parsed_memory_map!()
                .iter()
                .map(|x| x.length as usize)
                .sum::<usize>();
            uninit.write(Self::new(
                PhysicalAddress(PAGE_ALLOCATOR_OFFSET).translate(),
                memory_size,
            ));
            let start_address = const {
                PhysicalAddress::new_unchecked(FIRST_STAGE_OFFSET as usize)
                    .align_down(REGULAR_PAGE_ALIGNMENT)
            };
            let start_position = Self::address_position(start_address.clone()).unwrap();
            let initialized = uninit.assume_init_mut();
            println!("Initial Memory: {}", initialized.available_memory());
            // Allocate the addresses that are used for the code, and for other variables.
            let end_address = PhysicalAddress::new(
                PAGE_ALLOCATOR_OFFSET + (initialized.map().map.len() * size_of::<u64>()),
            )
            .align_up(REGULAR_PAGE_ALIGNMENT);
            let size_bits = ((end_address - start_address) / 0x1000).as_usize();
            let block = ContiguousBlockLayout::from_start_size(&start_position, size_bits);
            initialized
                .map_mut()
                .set_contiguous_block(&start_position, &block);
            parsed_memory_map!().iter().for_each(|region| {
                if region.region_type != MemoryRegionType::Usable {
                    let start_address_alligned = PhysicalAddress::new(
                        region.base_address as usize & (u64::MAX ^ (0x1000 - 1)) as usize,
                    );
                    let start_position = Self::address_position(start_address_alligned).unwrap();
                    let size_bits = region.length as usize / 0x1000;
                    let block = ContiguousBlockLayout::from_start_size(&start_position, size_bits);
                    initialized
                        .map_mut()
                        .set_contiguous_block(&start_position, &block);
                }
            });
        };
    }

    /// Converts a bitmap position to the corresponding physical address.
    ///
    /// The position is interpreted as an offset in the bitmap, where each bit represents a physical page.
    /// The resulting address is page-aligned and calculated based on the page size and bit position.
    ///
    /// # Examples
    ///
    /// ```
    /// let position = Position { map_index: 2, bit_index: 5 };
    /// let address = allocator.resolve_position(&position);
    /// assert_eq!(address.as_u64() % REGULAR_PAGE_SIZE as u64, 0);
    /// ```
    pub fn resolve_position(&self, p: &Position) -> PhysicalAddress {
        return PhysicalAddress::new(
            ((p.map_index * (u64::BITS as usize)) + p.bit_index as usize) * REGULAR_PAGE_SIZE,
        );
    }

    /// Returns the total amount of free physical memory in bytes.
    ///
    /// Calculates the number of unallocated pages by counting zero bits in the bitmap and multiplies by the regular page size.
    ///
    /// # Examples
    ///
    /// ```
    /// let free_bytes = allocator.available_memory();
    /// assert!(free_bytes % REGULAR_PAGE_SIZE == 0);
    /// ```
    pub fn available_memory(&self) -> usize {
        unsafe { self.map().count_zeros() * REGULAR_PAGE_SIZE }
    }

    /// Allocates a single physical page for use as a page table and returns a mutable reference to it.
    ///
    /// The allocated page is zero-initialized and marked as used in the bitmap.
    /// Panics if no free physical page is available.
    ///
    /// # Returns
    /// A mutable reference to the newly allocated and initialized `PageTable`.
    ///
    /// # Panics
    /// Panics if there are no free physical pages available.
    ///
    /// # Examples
    ///
    /// ```
    /// let table = allocator.alloc_table();
    /// // Use `table` as a mutable PageTable
    /// ```
    pub(super) fn alloc_table(&self) -> &'static mut PageTable {
        let free_block = unsafe { self.map().find_free_block(1) };

        match free_block {
            Some((p, _)) => unsafe {
                let physical_address = self.resolve_position(&p);

                ptr::write(
                    physical_address.translate().as_mut_ptr::<PageTable>(),
                    PageTable::empty(),
                );

                self.map_mut().set_bit(&p);

                return &mut *physical_address.as_mut_ptr::<PageTable>();
            },

            None => panic!("No physical memory is available to allocate this table"),
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl GlobalAlloc for PhysicalPageAllocator {
    /// Allocates a contiguous block of physical pages matching the specified layout.
    ///
    /// The allocation is aligned to the regular page size. Returns a pointer to the start of the allocated physical memory, or a null pointer if alignment fails or no suitable block is available.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the returned memory is used safely and that the allocator remains valid for the duration of use. Deallocation is not implemented.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::alloc::Layout;
    /// let layout = Layout::from_size_align(4096, 4096).unwrap();
    /// let ptr = unsafe { allocator.alloc(layout) };
    /// assert!(!ptr.is_null());
    /// ```
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match layout.align_to(REGULAR_PAGE_ALIGNMENT.as_usize()) {
            Ok(layout) => match self
                .map()
                .find_free_block(layout.size() / REGULAR_PAGE_SIZE)
            {
                Some((p, block)) => {
                    self.map_mut().set_contiguous_block(&p, &block);
                    self.resolve_position(&p).translate().as_mut_ptr::<u8>()
                }
                None => null::<u8>() as *mut u8,
            },
            Err(_) => null::<u8>() as *mut u8,
        }
    }

    /// Deallocation is not implemented for this allocator.
///
/// This method is a no-op and does not free any memory.
unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

unsafe impl Sync for PhysicalPageAllocator {}
