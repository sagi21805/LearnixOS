#![feature(const_trait_impl)]

use super::enums::{ProtectionLevel, SegmentDescriptorType};

#[repr(transparent)]
pub struct AccessByte(u8);
#[automatically_derived]
impl ::core::marker::Copy for AccessByte {}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for AccessByte {}
#[automatically_derived]
impl ::core::clone::Clone for AccessByte {
    #[inline]
    fn clone(&self) -> AccessByte {
        let _: ::core::clone::AssertParamIsClone<u8>;
        *self
    }
}
impl AccessByte {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }
    #[inline]
    #[track_caller]
    fn is_accessed(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 0usize;
            let bits = (val & mask) >> 0usize;
            <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                .expect("Cannot convert bit representation into bool")
        }
    }
    #[inline]
    #[track_caller]
    fn is_readable_writable(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 1usize;
            let bits = (val & mask) >> 1usize;
            <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                .expect("Cannot convert bit representation into bool")
        }
    }
    #[inline]
    #[track_caller]
    fn set_readable_writable(&mut self, v: bool) {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (bool) into u8");
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::set_readable_writable: value out of \
                         range: must fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 1usize;
            let cleared = val & !mask;
            let new = cleared | ((v as u8) << 1usize);
            ::core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    #[track_caller]
    const fn readable_writable(mut self, v: bool) -> Self {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (bool) into u8");
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::readable_writable: value out of \
                         range: must fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        self.0 |= (v as u8) << 1usize;
        self
    }
    #[inline]
    #[track_caller]
    fn is_direction_conforming(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 2usize;
            let bits = (val & mask) >> 2usize;
            <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                .expect("Cannot convert bit representation into bool")
        }
    }
    #[inline]
    #[track_caller]
    fn set_direction_conforming(&mut self, v: bool) {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (bool) into u8");
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::set_direction_conforming: value out \
                         of range: must fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 2usize;
            let cleared = val & !mask;
            let new = cleared | ((v as u8) << 2usize);
            ::core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    #[track_caller]
    const fn direction_conforming(mut self, v: bool) -> Self {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (bool) into u8");
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::direction_conforming: value out of \
                         range: must fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        self.0 |= (v as u8) << 2usize;
        self
    }
    #[inline]
    #[track_caller]
    fn is_executable(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 3usize;
            let bits = (val & mask) >> 3usize;
            <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                .expect("Cannot convert bit representation into bool")
        }
    }
    #[inline]
    #[track_caller]
    fn set_executable(&mut self, v: bool) {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (bool) into u8");
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::set_executable: value out of range: \
                         must fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 3usize;
            let cleared = val & !mask;
            let new = cleared | ((v as u8) << 3usize);
            ::core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    #[track_caller]
    const fn executable(mut self, v: bool) -> Self {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (bool) into u8");
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::executable: value out of range: \
                         must fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        self.0 |= (v as u8) << 3usize;
        self
    }
    #[inline]
    #[track_caller]
    fn get_segment_type(&self) -> SegmentDescriptorType {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 4usize;
            let bits = (val & mask) >> 4usize;
            <SegmentDescriptorType as ::core::convert::TryFrom<
                        u8,
                    >>::try_from(bits as u8)
                        .expect(
                            "Cannot convert bit representation into SegmentDescriptorType",
                        )
        }
    }
    #[inline]
    #[track_caller]
    fn set_segment_type(&mut self, v: SegmentDescriptorType) {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect(
                "Can't convert value 'v' (SegmentDescriptorType) into u8",
            );
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::set_segment_type: value out of \
                         range: must fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 4usize;
            let cleared = val & !mask;
            let new = cleared | ((v as u8) << 4usize);
            ::core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    #[track_caller]
    const fn segment_type(mut self, v: SegmentDescriptorType) -> Self {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect(
                "Can't convert value 'v' (SegmentDescriptorType) into u8",
            );
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::segment_type: value out of range: \
                         must fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        self.0 |= (v as u8) << 4usize;
        self
    }
    #[inline]
    #[track_caller]
    fn get_dpl(&self) -> ProtectionLevel {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 2usize as u32)) << 5usize;
            let bits = (val & mask) >> 5usize;
            <ProtectionLevel as ::core::convert::TryFrom<u8>>::try_from(
                bits as u8,
            )
            .expect(
                "Cannot convert bit representation into ProtectionLevel",
            )
        }
    }
    #[inline]
    #[track_caller]
    fn set_dpl(&mut self, v: ProtectionLevel) {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (ProtectionLevel) into u8");
        if true {
            if !((v as u8) <= (3u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::set_dpl: value out of range: must \
                         fit in 2 bits (max 0x3)",
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 2usize as u32)) << 5usize;
            let cleared = val & !mask;
            let new = cleared | ((v as u8) << 5usize);
            ::core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    #[track_caller]
    const fn dpl(mut self, v: ProtectionLevel) -> Self {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (ProtectionLevel) into u8");
        if true {
            if !((v as u8) <= (3u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::dpl: value out of range: must fit \
                         in 2 bits (max 0x3)",
                    ));
                }
            }
        }
        self.0 |= (v as u8) << 5usize;
        self
    }
    #[inline]
    #[track_caller]
    fn is_present(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 7usize;
            let bits = (val & mask) >> 7usize;
            <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                .expect("Cannot convert bit representation into bool")
        }
    }
    #[inline]
    #[track_caller]
    fn set_present(&mut self, v: bool) {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (bool) into u8");
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::set_present: value out of range: \
                         must fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = ::core::ptr::read_volatile(addr);
            let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 7usize;
            let cleared = val & !mask;
            let new = cleared | ((v as u8) << 7usize);
            ::core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    #[track_caller]
    const fn present(mut self, v: bool) -> Self {
        let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
            .ok()
            .expect("Can't convert value 'v' (bool) into u8");
        if true {
            if !((v as u8) <= (1u128 as u8)) {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "AccessByte::present: value out of range: must \
                         fit in 1 bits (max 0x1)",
                    ));
                }
            }
        }
        self.0 |= (v as u8) << 7usize;
        self
    }
}
impl const ::core::convert::From<u8> for AccessByte {
    fn from(value: u8) -> Self {
        AccessByte(value)
    }
}
impl const ::core::convert::From<AccessByte> for u8 {
    fn from(value: AccessByte) -> Self {
        value.0
    }
}
impl ::core::fmt::Debug for AccessByte {
    fn fmt(
        &self,
        f: &mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result {
        f.debug_struct("AccessByte")
            .field("accessed", &self.is_accessed())
            .field("readable_writable", &self.is_readable_writable())
            .field("direction_conforming", &self.is_direction_conforming())
            .field("executable", &self.is_executable())
            .field("segment_type", &self.get_segment_type())
            .field("dpl", &self.get_dpl())
            .field("present", &self.is_present())
            .finish()
    }
}
