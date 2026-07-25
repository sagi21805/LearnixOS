#[macro_export]
macro_rules! register_slabs {
    ($($t:ty),* $(,)?) => {
        $crate::register_slabs!(@step 0; $($t),*);
    };

    (@step $idx:expr; $head:ty, $($tail:ty),+) => {
        impl $crate::traits::SlabPosition for $head {
            const SLAB_POSITION: usize = $idx;
        }

        impl $crate::traits::Slab for $head {}

        $crate::register_slabs!(@step $idx + 1; $($tail),*);
    };

    (@step $idx:expr; $head:ty) => {
        impl $crate::traits::SlabPosition for $head {
            const SLAB_POSITION: usize = $idx;
        }

        impl $crate::traits::Slab for $head {}
    };

    (@step $idx:expr; ) => {};
}

#[macro_export]
macro_rules! define_slab_system {
    ($($t:ty),* $(,)?) => {
        use common::constants::REGULAR_PAGE_SIZE;
        use $crate::traits::SlabCacheConstructor;

        $crate::register_slabs!($($t),*);

        const COUNT: usize = [$(stringify!($t)),*].len();

        pub struct SlabAllocator {
            slabs: [common::late_init::LateInit<SlabCache<()>>; COUNT]
        }

        impl SlabAllocator {
            pub const fn new() -> Self {
                Self {
                    slabs: [
                        $({
                            let _ = stringify!($t);
                            common::late_init::LateInit::uninit()
                        }),*
                    ]
                }
            }

            pub fn init(&'static mut self) {
                $(
                    let index = <$t>::SLAB_POSITION;

                    let initialized = SlabCache::<$t>::new(size_of::<$t>().div_ceil(REGULAR_PAGE_SIZE));

                    let unassigned = NonNull::from_ref(&initialized).as_unassigned();

                    self.slabs[index].write(unsafe { unassigned.as_ref().clone() });
                )*
            }
        }
    }
}

// TODO implement reverse lookup with an enum that will automatically be
// generated and check the code generated on compiler explorer. if
// interesting, write on it on the book
