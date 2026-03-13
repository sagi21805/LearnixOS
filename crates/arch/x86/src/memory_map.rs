use common::{
    constants::{INIT_AREA_SIZE_BYTES, KiB, MiB},
    enums::MemoryRegionType,
};
use core::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    ptr::NonNull,
};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct MemoryRegionExtended {
    base_address: u64,
    length: u64,
    region_type: MemoryRegionType,
    extended_attributes: u32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct MemoryRegion {
    pub base_address: u64,
    pub length: u64,
    pub region_type: MemoryRegionType,
}

impl From<&MemoryRegionExtended> for MemoryRegion {
    fn from(value: &MemoryRegionExtended) -> Self {
        MemoryRegion {
            base_address: value.base_address,
            length: value.length,
            region_type: value.region_type,
        }
    }
}

#[derive(Debug)]
pub enum MemoryMapError {
    Empty,
    Overflow,
}

pub struct MemoryMap {
    pub regions: NonNull<[MemoryRegion]>,
    pub capacity: usize,
}

impl Deref for MemoryMap {
    type Target = [MemoryRegion];

    fn deref(&self) -> &Self::Target {
        unsafe { self.regions.as_ref() }
    }
}

impl Display for MemoryMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut usable = 0u64;
        let mut reserved = 0u64;

        for entry in self.iter() {
            let size_mib = entry.length / MiB as u64;
            let size_kib = (entry.length % MiB as u64) / KiB as u64;

            write!(
                f,
                "[0x{:0>9x} - 0x{:0>9x}]: type: {:?}",
                entry.base_address,
                entry.base_address + entry.length,
                entry.region_type,
            )?;

            match entry.region_type {
                MemoryRegionType::Usable
                | MemoryRegionType::Reserved
                | MemoryRegionType::Filler => {
                    if entry.region_type == MemoryRegionType::Usable {
                        usable += entry.length;
                    }
                    if entry.region_type == MemoryRegionType::Reserved {
                        reserved += entry.length;
                    }
                    writeln!(
                        f,
                        " (Size: {:>4} MiB {:>4} KiB)",
                        size_mib, size_kib
                    )?;
                }
                _ => writeln!(f)?,
            }
        }

        writeln!(f)?;
        writeln!(
            f,
            "Total Usable Memory:   {:>5} MiB {:>4} KiB",
            usable / MiB as u64,
            (usable % MiB as u64) / KiB as u64,
        )?;
        writeln!(
            f,
            "Total Reserved Memory: {:>5} MiB {:>4} KiB",
            reserved / MiB as u64,
            (reserved % MiB as u64) / KiB as u64,
        )
    }
}

impl MemoryMap {
    pub fn parse_map(
        raw: &mut [MemoryRegionExtended],
        buf: &mut [MemoryRegion],
    ) -> Result<MemoryMap, MemoryMapError> {
        let mut position = 0;
        let mut push =
            |region: MemoryRegion| -> Result<(), MemoryMapError> {
                if position >= buf.len() {
                    return Err(MemoryMapError::Overflow);
                }
                buf[position] = region;
                position += 1;
                Ok(())
            };

        let first_usable = raw
            .iter_mut()
            .find(|f| f.region_type == MemoryRegionType::Usable)
            .ok_or(MemoryMapError::Empty)?;

        assert!(first_usable.length > INIT_AREA_SIZE_BYTES);

        let init_entry = MemoryRegion {
            base_address: first_usable.base_address,
            length: INIT_AREA_SIZE_BYTES,
            region_type: MemoryRegionType::UserEnterd,
        };

        first_usable.base_address += INIT_AREA_SIZE_BYTES;
        first_usable.length -= INIT_AREA_SIZE_BYTES;

        push(init_entry)?;

        for (left, right) in raw.iter().map_windows(|[a, b]| (*a, *b)) {
            let filler = filler_entry(left, right);

            push(left.into())?;

            if let Some(f) = filler {
                push(f)?;
            }
        }

        let last = raw.last().ok_or(MemoryMapError::Empty)?;

        push(last.into())?;

        let capacity = buf.len();

        let modified = unsafe {
            core::slice::from_raw_parts_mut(buf.as_mut_ptr(), position)
        };

        Ok(MemoryMap {
            regions: NonNull::from_mut(modified),
            capacity,
        })
    }
}

/// Return a filler entry if there is a gap between A and B
#[inline]
fn filler_entry(
    left: &MemoryRegionExtended,
    right: &MemoryRegionExtended,
) -> Option<MemoryRegion> {
    assert!(left.base_address < right.base_address);

    (left.base_address + left.length < right.base_address).then(|| {
        let filler_base = left.base_address + left.length;
        let length = right.base_address - filler_base;

        MemoryRegion {
            base_address: filler_base,
            length,
            region_type: MemoryRegionType::Filler,
        }
    })
}
