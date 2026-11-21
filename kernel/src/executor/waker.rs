use core::{
    ptr,
    task::{RawWaker, RawWakerVTable, Waker},
};

use super::{ready::READY, slot::Slot};

impl Slot {
    /// A waker that marks this slot ready. It owns nothing, so cloning
    /// and dropping it cost nothing, and waking it is one `lock or`.
    pub fn waker(self) -> Waker {
        let data = ptr::without_provenance(self.index());
        // SAFETY: the vtable's functions only read the index back.
        unsafe { Waker::new(data, &VTABLE) }
    }
}

/// The functions of every slot waker, whose data is its slot's index.
const VTABLE: RawWakerVTable =
    RawWakerVTable::new(clone_raw, wake_raw, wake_raw, drop_raw);

/// Copies a waker, which is only an index.
///
/// # Safety
///
/// `data` must be the data of a slot waker, which is its slot's index.
unsafe fn clone_raw(data: *const ()) -> RawWaker {
    RawWaker::new(data, &VTABLE)
}

/// Marks the waker's slot ready. Serves `wake` and `wake_by_ref`.
///
/// # Safety
///
/// `data` must be the data of a slot waker, which is its slot's index.
unsafe fn wake_raw(data: *const ()) { READY.wake(Slot::new(data.addr())) }

/// Drops a waker, which owns nothing.
///
/// # Safety
///
/// `data` must be the data of a slot waker, which is its slot's index.
unsafe fn drop_raw(_: *const ()) {}
