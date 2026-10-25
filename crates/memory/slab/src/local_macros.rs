/// Declares a generic allocation size for each `Name = size`, smallest
/// first, in the [`Generic`] group, `number_generic_slabs!`, which numbers
/// them before the slab types it is given, and `match_generic!`, which
/// runs a block with a type named after the smallest size that holds a
/// given number of bytes.
///
/// `define_generics!(Generic8 = 8, Generic16 = 16)` ->
/// `match_generic!(size, T => { body }, _ => fallback)` expands to
/// `match size { ..=8 => { type T = Generic8; body } ..=16 => { type T =
/// Generic16; body } _ => fallback }`.
///
/// [`Generic`]: crate::Generic
#[macro_export]
macro_rules! define_generics {
    ($($name:ident = $size:literal),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy)]
            #[::typestate_groups::state]
            pub struct $name(pub [usize; $size / 8]);
        )*

        /// The [`SlabFamily`](crate::traits::SlabFamily) group of the
        /// generic sizes.
        mod generic_family {
            use super::{$($name),*};
            use $crate::traits::{SlabFamily, SlabFamilyGroupMarker};

            /// The generic sizes, mapped as regular memory.
            #[::typestate_groups::group(Generic)]
            impl SlabFamily for ($($name,)*) {
                const PFLAGS: ::x86::structures::paging::PageEntryFlags =
                    ::x86::structures::paging::PageEntryFlags::regular_page_flags();
                const PSIZE: ::common::enums::PageSize =
                    ::common::enums::PageSize::Regular;
            }
        }

        pub use generic_family::Generic;

        /// Numbers the generic sizes and then the given slab types, as
        /// `number_slab_types!` does.
        macro_rules! number_generic_slabs {
            ($$($$t:ty),* $$(,)?) => {
                $crate::number_slab_types!($($name,)* $$($$t),*);
            };
        }

        /// Runs `$body` with `$t` naming the smallest generic size of at
        /// least `$size` bytes, or `$fallback` when none is that large.
        macro_rules! match_generic {
            (
                $$size:expr,
                $$t:ident => $$body:block,
                _ => $$fallback:expr
            ) => {
                match $$size {
                    $( ..=$size => { type $$t = $name; $$body } )*
                    _ => $$fallback,
                }
            };
        }
    };
}

/// Declares a DMA allocation size for each `Name = size`, smallest first,
/// in the [`DmaGeneric`] group, `define_slab_system!`, which numbers them
/// and the generic sizes before the slab types it is given, and
/// `match_dma_generic!`, which works as `match_generic!` does over the DMA
/// sizes.
///
/// Expands after `define_generics!`, as `define_slab_system!` calls the
/// `number_generic_slabs!` it declares.
///
/// `define_dma_generics!(DmaGeneric512 = 512)` ->
/// `define_slab_system!(Pcb)` expands to
/// `number_generic_slabs!(DmaGeneric512, Pcb)`.
///
/// [`DmaGeneric`]: crate::DmaGeneric
#[macro_export]
macro_rules! define_dma_generics {
    ($($name:ident = $size:literal),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy)]
            #[::typestate_groups::state]
            pub struct $name(pub [usize; $size / 8]);
        )*

        /// The [`SlabFamily`](crate::traits::SlabFamily) group of the DMA
        /// sizes.
        mod dma_generic_family {
            use super::{$($name),*};
            use $crate::traits::{SlabFamily, SlabFamilyGroupMarker};

            /// The DMA sizes, mapped uncached so a device and the CPU see
            /// the same bytes.
            #[::typestate_groups::group(DmaGeneric)]
            impl SlabFamily for ($($name,)*) {
                const PFLAGS: ::x86::structures::paging::PageEntryFlags =
                    ::x86::structures::paging::PageEntryFlags::regular_io_page_flags();
                const PSIZE: ::common::enums::PageSize =
                    ::common::enums::PageSize::Regular;
            }
        }

        pub use dma_generic_family::DmaGeneric;

        /// Numbers the DMA sizes, the generic sizes and then the given
        /// slab types, as `number_slab_types!` does.
        macro_rules! define_slab_system {
            ($$($$t:ty),* $$(,)?) => {
                number_generic_slabs!($($name,)* $$($$t),*);
            };
        }

        /// Runs `$body` with `$t` naming the smallest DMA size of at least
        /// `$size` bytes, or `$fallback` when none is that large.
        macro_rules! match_dma_generic {
            (
                $$size:expr,
                $$t:ident => $$body:block,
                _ => $$fallback:expr
            ) => {
                match $$size {
                    $( ..=$size => { type $$t = $name; $$body } )*
                    _ => $$fallback,
                }
            };
        }
    };
}

/// Numbers the slab types in order, giving each its position in
/// [`SlabCaches`](crate::cache::SlabCaches), and counts them in
/// `SLAB_COUNT`.
///
/// `number_slab_types!(Generic8, Generic16)` ->
/// `Generic8::SLAB_POSITION == 0`, `Generic16::SLAB_POSITION == 1`, and
/// `SLAB_COUNT == 2`.
#[macro_export]
macro_rules! number_slab_types {
    ($($t:ty),* $(,)?) => {
        $(
            impl $crate::traits::sealed::Sealed for $t {}

            impl $crate::traits::SlabGeometry for $t {}

            impl $crate::traits::Slab for $t {
                const SLAB_POSITION: usize = ${index()};
            }
        )*

        /// The number of slab types.
        const SLAB_COUNT: usize = ${count($t)};
    };
}
