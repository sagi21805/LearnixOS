fn generate_mask_1(n: u8) -> u8 {
    (1 << n) - 1
}

fn generate_mask_2(n: u8) -> u8 {
    u8::MAX >> (u8::BITS - n as u32)
}

fn generate_mask_3(n: u8, offset: u8) -> u8 {
    (u8::MAX >> (u8::BITS - n as u32)) << offset
}

fn main() {
    // Change n to the current mask!
    let mask = generate_mask_n(3);
    println!("0b{:08b}", mask);
}
