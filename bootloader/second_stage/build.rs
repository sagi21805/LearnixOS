use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));

    println!("cargo::rerun-if-changed={:?}/src", local_path);
    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        local_path.join("32bit.ld").display()
    )
}
