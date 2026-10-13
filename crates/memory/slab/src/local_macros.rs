/// Declares a [`Generic`](crate::traits::Generic) size class for each
/// `Name = size`, covering the sizes above the previous class up to its
/// own.
///
/// `define_generics!(Generic8 = 8, Generic16 = 16)` -> `Generic8` covers
/// `0..=8` and `Generic16` covers `9..=16`.
#[macro_export]
macro_rules! define_generics {
    ($($name:ident = $size:literal),* $(,)?) => {
        $crate::define_generics!(@step 0; $($name = $size),*);
    };
    (@step $start:expr; $name:ident = $size:literal $(, $rest:ident = $rest_size:literal)*) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name(pub [usize; $size / 8]);

        impl $crate::traits::Generic for $name {
            const START: usize = $start;
            const END: usize = $size;

            fn size(&self) -> usize { $size }
        }

        $crate::define_generics!(@step $size + 1; $($rest = $rest_size),*);
    };
    (@step $start:expr;) => {};
}

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
