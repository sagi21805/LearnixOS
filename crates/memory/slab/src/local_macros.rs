/// Declares the generic allocation sizes, smallest first, in the
/// [`Generic`] group, along with their caches in `GenericCaches` and
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

        $crate::slab_geometry!($($name),*);
        $crate::define_builtin_caches!(GenericCaches, generic: $($name),*);

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
/// [`DmaGeneric`] group, along with their caches in `DmaGenericCaches`
/// and `match_dma_generic!` over them.
///
/// ```ignore
/// define_dma_generics!(DmaGeneric512 = 512);
///
/// match_dma_generic!(size, T => { body }, _ => fallback)
/// // expands to
/// match size {
///     ..=512 => { type T = DmaGeneric512; body }
///     _ => fallback,
/// }
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

        $crate::slab_geometry!($($name),*);
        $crate::define_builtin_caches!(DmaGenericCaches, dma: $($name),*);

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

/// The tuple struct `$caches` with one cache per built-in size, and
/// `CachedIn` for each size in the caches of every program, through field
/// `$field` of [`BuiltinCaches`](crate::cache::BuiltinCaches).
///
/// ```ignore
/// define_builtin_caches!(GenericCaches, generic: Generic8, Generic16);
/// // expands to
/// pub(crate) struct GenericCaches(SlabCache<Generic8>, SlabCache<Generic16>);
/// unsafe impl<C> CachedIn<C> for Generic8 { .. caches.builtin.generic.0 }
/// unsafe impl<C> CachedIn<C> for Generic16 { .. caches.builtin.generic.1 }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! define_builtin_caches {
    ($caches:ident, $field:ident: $($name:ident),*) => {
        pub(crate) struct $caches($($crate::cache::SlabCache<$name>),*);

        impl $caches {
            /// Every cache, empty.
            pub(crate) const EMPTY: Self = Self(
                $({ stringify!($name); $crate::cache::SlabCache::new() }),*
            );
        }

        $(
            // SAFETY: field `${index()}` of the built-in caches holds the
            // cache of `$name`, and no other type reaches it.
            unsafe impl<C> $crate::cache::CachedIn<C> for $name {
                fn cache(
                    caches: &mut $crate::cache::SlabCaches<C>,
                ) -> &mut $crate::cache::SlabCache<Self> {
                    &mut caches.builtin.$field.${index()}
                }
            }
        )*
    };
}

/// [`SlabGeometry`](crate::traits::SlabGeometry) for each of the types
/// `$t`.
///
/// ```ignore
/// slab_geometry!(Inode);
/// // expands to
/// unsafe impl SlabGeometry for Inode {
///     type PreAllocatedArray =
///         [PreAllocated<Inode>; Inode::OBJECTS_PER_SLAB];
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! slab_geometry {
    ($($t:ty),* $(,)?) => {
        $(
            // SAFETY: the array holds `OBJECTS_PER_SLAB` slots of `$t`.
            unsafe impl $crate::traits::SlabGeometry for $t {
                type PreAllocatedArray = [
                    $crate::preallocated::PreAllocated<$t>;
                    <$t as $crate::traits::SlabGeometry>::OBJECTS_PER_SLAB
                ];
            }
        )*
    };
}
