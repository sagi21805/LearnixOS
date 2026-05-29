use std::path::Path;

fn main() {
    // Environment variable that stores the current working directory
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));

    // This tells cargo to add the `-C link-arg=--script=./linker.ld`
    // argument. Which will result in linking with our code with our
    // linker script
    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        local_path.join("linker.ld").display()
    )
}
