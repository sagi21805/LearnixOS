// Note that currently this function does not make bound checcks for our
// value.
fn write_flag(value: u8, offset: u8, width: u8, new_value: u8) -> u8 {
    let mask = !generate_mask_3(width, offset);
    let cleared = value & mask;
    let shifted = (new_value as u8) << offset;
    cleared | shifted
}

fn main() {
    let value = 0b11011011;

    let offset = 2;
    let width = 3;
    let new_value = 2;
    let read = write_flag(value, offset, width, new_value);
    println!("0b{:08b}", read);
}
