use crate::drivers::vga_display::color_code::{Color, ColorCode};
use crate::{print, println};

#[cfg(feature = "test")]
const TESTS: &[&dyn Fn()] = &[&example_test, &example_test, &example_test, &example_test];

#[cfg(feature = "test")]
pub fn test_runner() {
    let test_num = TESTS.len();
    println!("Running {} tests", test_num);

    for (i, test) in TESTS.iter().enumerate() {
        println!("Running Test Number: {}", i);
        test();
        print!("Status: [");
        print!("ok" ; color = ColorCode::new(Color::Green, Color::Black));
        println!("]\n");
    }

    loop {}
}

#[cfg(feature = "test")]
fn example_test() {
    println!("Example test");
}
