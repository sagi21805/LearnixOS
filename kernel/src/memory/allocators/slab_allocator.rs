use common::address_types::PhysicalAddress;



pub struct Block {
    address: [PhysicalAddress], // address of the page.
    next: Option<&'static Block>
}

pub struct SlabAllocator {
    pages: [&]
}