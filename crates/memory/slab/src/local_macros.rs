#[macro_export]
macro_rules! register_slabs {
    ($($t:ty),* $(,)?) => {
        $crate::register_slabs!(@step 0; $($t),*);
    };
    // One arm handles both "head + rest" and "head only" via optional repetition.
    (@step $idx:expr; $head:ty $(, $tail:ty)*) => {
        impl $crate::traits::SlabPosition for $head {
            const SLAB_POSITION: usize = $idx;
        }

        impl $crate::traits::Slab for $head {}

        $crate::register_slabs!(@step $idx + 1; $($tail),*);
    };
    (@step $idx:expr; ) => {};
}

#[macro_export]
macro_rules! define_slab_system {
    ($($t:ty),* $(,)?) => {
        use common::constants::REGULAR_PAGE_SIZE;

        $crate::register_slabs!($($t),*);

        const COUNT: usize = [$(stringify!($t)),*].len();

        const EMPTY_SLAB: SlabCache<()> = SlabCache::default();

        pub(crate) static SLAB_ARENA: SpinMutex<[SlabCache<()>; COUNT]> = SpinMutex::new([EMPTY_SLAB; COUNT]);

        impl<Block, Arena> SlabAllocator<Block, Arena> where
            Block: BuddyBlock + SlabBlock,
            Arena: BuddyArena<Block>
        {


            pub fn init(&mut self) {
                $(
                    let index = <$t>::SLAB_POSITION;

                    let slab_cache = SlabCache::<$t>::new(size_of::<$t>().div_ceil(REGULAR_PAGE_SIZE));

                    self.slab_arena.lock()[index] = unsafe { slab_cache.as_unit() };
                )*
            }
        }
    }
}

// TODO implement reverse lookup with an enum that will automatically be
// generated and check the code generated on compiler explorer. if
// interesting, write on it on the book
