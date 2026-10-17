/// Declares a [`Generic`](crate::traits::Generic) size class for each
/// `Name = size`, covering the sizes above the previous class up to its
/// own, plus a `SizeClass` enum of them and the `Allocator` impl of
/// `SlabAllocator` that picks a class per layout.
///
/// `define_generics!(Generic8 = 8, Generic16 = 16)` -> `Generic8` covers
/// `0..=8` and `Generic16` covers `9..=16`.
#[macro_export]
macro_rules! define_generics {
    ($($name:ident = $size:literal),* $(,)?) => {
        $crate::define_generics!(@step 0; $($name = $size),*);

        /// The generic size classes, smallest first.
        #[derive(Debug, Clone, Copy)]
        pub enum SizeClass {
            $($name),*
        }

        impl SizeClass {
            /// The smallest class whose objects fit `layout` in size and
            /// alignment.
            pub fn of(layout: core::alloc::Layout) -> Option<SizeClass> {
                $(
                    if layout.size() <= <$name as $crate::traits::Generic>::END
                        && layout.align()
                            <= <$name as $crate::traits::SlabPosition>::OBJECT_ALIGNMENT
                    {
                        return Some(SizeClass::$name);
                    }
                )*
                None
            }
        }

        // SAFETY: each layout maps to one size class, and every class
        // forwards to its `CacheAllocator`, so blocks come from and return
        // to the same cache.
        unsafe impl<Block, Arena> core::alloc::Allocator
            for $crate::SlabAllocator<Block, Arena>
        where
            Block: buddy::meta::BuddyBlock + $crate::traits::SlabBlock,
            Arena: buddy::meta::BuddyArena<Block> + 'static,
        {
            fn allocate(
                &self,
                layout: core::alloc::Layout,
            ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
                match SizeClass::of(layout).ok_or(core::alloc::AllocError)? {
                    $(SizeClass::$name => self.cache::<$name>().allocate(layout),)*
                }
            }

            unsafe fn deallocate(
                &self,
                ptr: core::ptr::NonNull<u8>,
                layout: core::alloc::Layout,
            ) {
                let class = SizeClass::of(layout);
                debug_assert!(class.is_some(), "`layout` came from `allocate`");
                // SAFETY: `layout` fits a block `allocate` returned, so `of`
                // found a class then and finds the same one now.
                match unsafe { class.unwrap_unchecked() } {
                    $(
                        // SAFETY: the caller's guarantees carry over to the
                        // cache that allocated `ptr`.
                        SizeClass::$name => unsafe {
                            self.cache::<$name>().deallocate(ptr, layout)
                        },
                    )*
                }
            }
        }
    };
    (@step $start:expr; $name:ident = $size:literal $(, $rest:ident = $rest_size:literal)*) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name(pub [usize; $size / 8]);

        impl $crate::traits::Generic for $name {
            const START: usize = $start;
            const END: usize = $size;
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

                    let slab_cache = SlabCache::<$t>::new();

                    self.slab_arena.lock()[index] = unsafe { slab_cache.as_unit() };
                )*
            }
        }
    }
}

// TODO implement reverse lookup with an enum that will automatically be
// generated and check the code generated on compiler explorer. if
// interesting, write on it on the book
