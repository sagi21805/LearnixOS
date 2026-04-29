#![feature(const_trait_impl)]

use super::enums::{ProtectionLevel, SegmentDescriptorType};

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
    pub fn new() -> Self {
        Self(0)
    }
    #[inline]
    fn is_accessed(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            val & (1 << 0usize) != 0
        }
    }
    #[inline]
    fn is_readable_writable(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            val & (1 << 1usize) != 0
        }
    }
    #[inline]
    fn set_readable_writable(&mut self, v: bool) {
        if false {
            if !((u8::try_from(v)
                .ok()
                .expect("Can't convery value 'v' into the struct type")
                as u8)
                < (1 << 1usize) as u8)
            {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "Value: {0:?} is too large for this bitfield",
                        v,
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            let cleared = val & !(((1 << 1usize) - 1) << 1usize);
            let new =
                cleared | ((u8::try_from(v).unwrap() as u8) << 1usize);
            core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    const fn readable_writable(mut self) -> Self {
        self.0 |= (1 << 1usize);
        self
    }
    #[inline]
    fn is_direction_conforming(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            val & (1 << 2usize) != 0
        }
    }
    #[inline]
    fn set_direction_conforming(&mut self, v: bool) {
        if false {
            if !((u8::try_from(v)
                .ok()
                .expect("Can't convery value 'v' into the struct type")
                as u8)
                < (1 << 1usize) as u8)
            {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "Value: {0:?} is too large for this bitfield",
                        v,
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            let cleared = val & !(((1 << 1usize) - 1) << 2usize);
            let new =
                cleared | ((u8::try_from(v).unwrap() as u8) << 2usize);
            core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    const fn direction_conforming(mut self) -> Self {
        self.0 |= (1 << 2usize);
        self
    }
    #[inline]
    fn is_executable(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            val & (1 << 3usize) != 0
        }
    }
    #[inline]
    fn set_executable(&mut self, v: bool) {
        if false {
            if !((u8::try_from(v)
                .ok()
                .expect("Can't convery value 'v' into the struct type")
                as u8)
                < (1 << 1usize) as u8)
            {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "Value: {0:?} is too large for this bitfield",
                        v,
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            let cleared = val & !(((1 << 1usize) - 1) << 3usize);
            let new =
                cleared | ((u8::try_from(v).unwrap() as u8) << 3usize);
            core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    const fn executable(mut self) -> Self {
        self.0 |= (1 << 3usize);
        self
    }
    #[inline]
    fn get_segment_type(&self) -> SegmentDescriptorType {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            SegmentDescriptorType::try_from(
                ((val >> 4usize) & ((1 << 1usize) - 1)) as u8,
            )
            .expect(
                "Cannot convert bit representation into the given type",
            )
        }
    }
    #[inline]
    fn set_segment_type(&mut self, v: bool) {
        if false {
            if !((u8::try_from(v)
                .ok()
                .expect("Can't convery value 'v' into the struct type")
                as u8)
                < (1 << 1usize) as u8)
            {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "Value: {0:?} is too large for this bitfield",
                        v,
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            let cleared = val & !(((1 << 1usize) - 1) << 4usize);
            let new =
                cleared | ((u8::try_from(v).unwrap() as u8) << 4usize);
            core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    const fn segment_type(mut self, v: SegmentDescriptorType) -> Self {
        if false {
            if !((u8::try_from(v)
                .ok()
                .expect("Can't convery value 'v' into the struct type")
                as u8)
                < (1 << 1usize) as u8)
            {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "Value is too large for this bitfield"
                    ));
                }
            }
        }
        self.0 |= ((u8::try_from(v)
            .ok()
            .expect("Can't convery value 'v' into the struct type")
            as u8)
            << 4usize);
        self
    }
    #[inline]
    fn get_dpl(&self) -> ProtectionLevel {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            ProtectionLevel::try_from(
                ((val >> 5usize) & ((1 << 2usize) - 1)) as u8,
            )
            .expect(
                "Cannot convert bit representation into the given type",
            )
        }
    }
    #[inline]
    fn set_dpl(&mut self, v: ProtectionLevel) {
        if false {
            if !((u8::try_from(v)
                .ok()
                .expect("Can't convery value 'v' into the struct type")
                as u8)
                < (1 << 2usize) as u8)
            {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "Value: {0:?} is too large for this bitfield",
                        v,
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            let cleared = val & !(((1 << 2usize) - 1) << 5usize);
            let new =
                cleared | ((u8::try_from(v).unwrap() as u8) << 5usize);
            core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    const fn dpl(mut self, v: ProtectionLevel) -> Self {
        if false {
            if !((u8::try_from(v)
                .ok()
                .expect("Can't convery value 'v' into the struct type")
                as u8)
                < (1 << 2usize) as u8)
            {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "Value is too large for this bitfield"
                    ));
                }
            }
        }
        self.0 |= ((u8::try_from(v)
            .ok()
            .expect("Can't convery value 'v' into the struct type")
            as u8)
            << 5usize);
        self
    }
    #[inline]
    fn is_present(&self) -> bool {
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            val & (1 << 7usize) != 0
        }
    }
    #[inline]
    fn set_present(&mut self, v: bool) {
        if false {
            if !((u8::try_from(v)
                .ok()
                .expect("Can't convery value 'v' into the struct type")
                as u8)
                < (1 << 1usize) as u8)
            {
                {
                    ::core::panicking::panic_fmt(format_args!(
                        "Value: {0:?} is too large for this bitfield",
                        v,
                    ));
                }
            }
        }
        unsafe {
            let addr = self as *const _ as *mut u8;
            let val = core::ptr::read_volatile(addr);
            let cleared = val & !(((1 << 1usize) - 1) << 7usize);
            let new =
                cleared | ((u8::try_from(v).unwrap() as u8) << 7usize);
            core::ptr::write_volatile(addr, new);
        }
    }
    #[inline]
    const fn present(mut self) -> Self {
        self.0 |= (1 << 7usize);
        self
    }
}
impl const Default for AccessByte {
    fn default() -> Self {
        Self(0)
    }
}
impl const From<u8> for AccessByte {
    fn from(value: u8) -> Self {
        AccessByte(value)
    }
}
impl const From<AccessByte> for u8 {
    fn from(value: AccessByte) -> u8 {
        value.0
    }
}
impl core::fmt::Debug for AccessByte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AccessByte")
            .field("accessed", &u8::try_from(self.is_accessed()))
            .field(
                "readable_writable",
                &u8::try_from(self.is_readable_writable()),
            )
            .field(
                "direction_conforming",
                &u8::try_from(self.is_direction_conforming()),
            )
            .field("executable", &u8::try_from(self.is_executable()))
            .field(
                "segment_type",
                &SegmentDescriptorType::try_from(self.get_segment_type()),
            )
            .field("dpl", &ProtectionLevel::try_from(self.get_dpl()))
            .field("present", &u8::try_from(self.is_present()))
            .finish()
    }
}
