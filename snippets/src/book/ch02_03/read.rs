fn read_flag(value: u8, offset: u8, width: u8) -> u8 {
    let mask = generate_mask_3(width, offset);

    ((value & mask) >> offset) as u8
}

fn main() {
    let value = 0b11011011;

    let offset = 2;
    let width = 3;
    let read = read_flag(value, offset, width);
    println!("{}", read);
}
