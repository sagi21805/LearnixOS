use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));

    println!("cargo::rerun-if-changed={:?}/src", local_path);
    println!(
        "cargo::rerun-if-changed=../../../build/linker_scripts/16bit.ld"
    );
    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        local_path
            .join("../../../build/linker_scripts/16bit.ld")
            .display()
    )
}
