use core::{
    pin::Pin,
    task::{Context, Poll},
};

use slab::{preallocated::PreAllocated, traits::SlabGeometry};

/// Every task the kernel runs, each in a box from its own slab cache.
pub enum Task {}

// `#[typestate_groups::state]` takes only structs.
impl typestate_groups::State for Task {}

impl Future for Task {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        // SAFETY: the variant's future stays where `self` is.
        match *unsafe { self.get_unchecked_mut() } {}
    }
}

slab::slab_family!(KernelTasks: Task);

// A new variant can't push the slab's waste above 1/8.
const _: () = {
    let used = Task::OBJECTS_PER_SLAB * size_of::<PreAllocated<Task>>();
    assert!(
        used * 8 >= Task::SLAB_LAYOUT.size() * 7,
        "a task's slab wastes at most 1/8 of its pages"
    );
};
