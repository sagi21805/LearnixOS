/// Declares the generic allocation sizes, smallest first, in the
/// [`Generic`] group, along with `number_generic_slabs!` and
/// `match_generic!` over them.
///
/// ```ignore
/// define_generics!(Generic8 = 8, Generic16 = 16);
///
/// match_generic!(size, T => { body }, _ => fallback)
/// // expands to
/// match size {
///     ..=8 => { type T = Generic8; body }
///     ..=16 => { type T = Generic16; body }
///     _ => fallback,
/// }
/// ```
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

        mod generic_family {
            use super::{$($name),*};
            use $crate::traits::{SlabFamily, SlabFamilyGroupMarker};

            #[::typestate_groups::group(Generic)]
            impl SlabFamily for ($($name,)*) {
                const PFLAGS: ::x86::structures::paging::PageEntryFlags =
                    ::x86::structures::paging::PageEntryFlags::regular_page_flags();
                const PSIZE: ::common::enums::PageSize =
                    ::common::enums::PageSize::Regular;
            }
        }

        pub use generic_family::Generic;

        /// `number_slab_types!` with the generic sizes first.
        macro_rules! number_generic_slabs {
            ($$($$t:ty),* $$(,)?) => {
                $crate::number_slab_types!($($name,)* $$($$t),*);
            };
        }

        /// Runs `$body` with `$t` as the smallest generic size that holds
        /// `$size` bytes.
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

/// Declares the DMA allocation sizes, smallest first, in the
/// [`DmaGeneric`] group, along with `define_slab_system!` and
/// `match_dma_generic!` over them. Call it after `define_generics!`.
///
/// ```ignore
/// define_dma_generics!(DmaGeneric512 = 512);
///
/// define_slab_system!(Pcb);
/// // expands to
/// number_generic_slabs!(DmaGeneric512, Pcb);
/// ```
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

        mod dma_generic_family {
            use super::{$($name),*};
            use $crate::traits::{SlabFamily, SlabFamilyGroupMarker};

            /// Mapped uncached, so a device and the CPU see the same bytes.
            #[::typestate_groups::group(DmaGeneric)]
            impl SlabFamily for ($($name,)*) {
                const PFLAGS: ::x86::structures::paging::PageEntryFlags =
                    ::x86::structures::paging::PageEntryFlags::regular_io_page_flags();
                const PSIZE: ::common::enums::PageSize =
                    ::common::enums::PageSize::Regular;
            }
        }

        pub use dma_generic_family::DmaGeneric;

        /// `number_generic_slabs!` with the DMA sizes first.
        macro_rules! define_slab_system {
            ($$($$t:ty),* $$(,)?) => {
                number_generic_slabs!($($name,)* $$($$t),*);
            };
        }

        /// `match_generic!` over the DMA sizes.
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
/// [`SlabCaches`](crate::cache::SlabCaches) and its `PreAllocatedArray`.
///
/// ```ignore
/// number_slab_types!(Generic8, Generic16);
///
/// assert_eq!(Generic8::SLAB_POSITION, 0);
/// assert_eq!(Generic16::SLAB_POSITION, 1);
/// assert_eq!(SLAB_COUNT, 2);
/// // Generic8::PreAllocatedArray = [PreAllocated<Generic8>; 512]
/// ```
#[macro_export]
macro_rules! number_slab_types {
    ($($t:ty),* $(,)?) => {
        $(
            impl $crate::traits::sealed::Sealed for $t {}

            impl $crate::traits::SlabGeometry for $t {
                type PreAllocatedArray = [
                    $crate::preallocated::PreAllocated<$t>;
                    <$t as $crate::traits::SlabGeometry>::OBJECT_PER_SLAB
                ];
            }

            impl $crate::traits::Slab for $t {
                const SLAB_POSITION: usize = ${index()};
            }
        )*

        /// The number of slab types.
        const SLAB_COUNT: usize = ${count($t)};
    };
}
