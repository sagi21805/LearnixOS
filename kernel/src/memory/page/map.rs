use core::ops::{Deref, DerefMut};

use common::late_init::LateInit;

use crate::memory::{memory_map::ParsedMemoryMap, page::UnassignedPage};

pub struct PageMap {
    map: &'static mut [UnassignedPage],
    // lock: todo!(),
}

impl Deref for PageMap {
    type Target = [UnassignedPage];

    fn deref(&self) -> &Self::Target {
        self.map
    }
}

impl DerefMut for PageMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.map
    }
}

impl PageMap {
    ///  Initializes all pages on a constant address.
    pub fn init(
        uninit: &'static mut LateInit<PageMap>,
        mmap: ParsedMemoryMap,
    ) {
        todo!()
    }
}
