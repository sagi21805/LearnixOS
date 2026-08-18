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

#[macro_export]
macro_rules! new_state {
    (
        head => $head: ident,
        attached => $attached: ident,
        detached => $detached: ident,
        meta => $meta: ty
        $(,)?
    ) => {
        pub struct $head;

        unsafe impl<T: Slab> $crate::traits::SlabState<T> for $head {
            type Meta = $meta;
        }

        pub struct $attached;

        unsafe impl<T: Slab> $crate::traits::SlabState<T> for $attached {
            type Meta = $meta;
        }

        unsafe impl<T: Slab> $crate::traits::AttachedSlabState<T>
            for $attached
        {
            type DetachedState = $detached;
            type HeadState = $head;
        }

        pub struct $detached;

        unsafe impl<T: Slab> $crate::traits::SlabState<T> for $detached {
            type Meta = $meta;
        }

        unsafe impl<T: Slab> $crate::traits::DetachedSlabState<T>
            for $detached
        {
            type AttachedState = $attached;
            type HeadState = $head;
        }

        unsafe impl<T: Slab> $crate::traits::AttachedSlab<T, $attached>
            for SlabDescriptor<T, $attached>
        {
        }

        unsafe impl<T: Slab> $crate::traits::DetachedSlab<T, $detached>
            for SlabDescriptor<T, $detached>
        {
        }

        unsafe impl<T: Slab> $crate::traits::HeadSlabState<T> for $head {
            type AttachedState = $attached;
            type DetachedState = $detached;
        }
    };
}

// TODO implement reverse lookup with an enum that will automatically be
// generated and check the code generated on compiler explorer. if
// interesting, write on it on the book
