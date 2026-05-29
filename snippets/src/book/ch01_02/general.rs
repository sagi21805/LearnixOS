use core::mem::MaybeUninit;

static mut MESSAGE: &'static str = "Hello World!";

static mut UNINIT: MaybeUninit<String> = MaybeUninit::uninit();

static VAR: u32 = 42;

fn some_function(x: u32, y: u32) -> u32 {
    return x + y;
}
