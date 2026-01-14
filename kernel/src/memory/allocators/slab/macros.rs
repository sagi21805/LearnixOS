#[macro_export]
macro_rules! register_slabs {
    ($($t:ty),* $(,)?) => {
        $crate::register_slabs!(@step 0; $($t),*);
    };

    (@step $idx:expr; $head:ty, $($tail:ty),+) => {
        impl $crate::slab::traits::SlabPosition for $head {
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

        $crate::register_slabs!($($t),*);

        const COUNT: usize = [$(stringify!($t)),*].len();

        pub static mut SLABS: [

            common::late_init::LateInit<SlabCache<$crate::memory::page_descriptor::Unassigned>>; COUNT] = [
            $(
                {
                    stringify!($t);
                    common::late_init::LateInit::uninit()
                }
            ),*
        ];
    }
}
