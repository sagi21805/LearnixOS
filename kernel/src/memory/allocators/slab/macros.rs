#[macro_export]
macro_rules! register_slabs {
    ($($t:ty),* $(,)?) => {
        $crate::register_slabs!(@step 0; $($t),*);
    };

    (@step $idx:expr; $head:ty, $($tail:ty),+) => {
        impl $crate::memory::allocators::slab::traits::SlabPosition for $head {
            const POSITION: usize = $idx;
        }
        $crate::register_slabs!(@step $idx + 1; $($tail),*);
    };

    (@step $idx:expr; $head:ty) => {
        impl $crate::memory::allocators::slab::traits::SlabPosition for $head {
            const POSITION: usize = $idx;
        }
    };

    (@step $idx:expr; ) => {};
}

#[macro_export]
macro_rules! define_slab_system {
    ($($t:ty),* $(,)?) => {
        use common::constants::REGULAR_PAGE_SIZE;

        $crate::register_slabs!($($t),*);

        const COUNT: usize = [$(stringify!($t)),*].len();

        pub struct SlabAllocator {
            slabs: [common::late_init::LateInit<SlabCache<$crate::memory::page_descriptor::Unassigned>>; COUNT]
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
                    let index = <$t>::POSITION;

                    self.slabs[index].write(SlabCache::<$t>::new(
                        (size_of::<$t>().next_multiple_of(REGULAR_PAGE_SIZE) / REGULAR_PAGE_SIZE) - 1
                    ).as_unassigned().clone());
                )*
            }
        }
    }
}

// TODO implement reverse lookup with an enum that will automatically be
// generated and check the code generated on compiler explorer. if
// interesting, write on it on the book
