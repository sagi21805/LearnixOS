use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    
    if std::env::var("CARGO_FEATURE_16BIT").unwrap() != "" {
        println!(
            "cargo:rustc-link-arg-bins=--script={}",
            local_path.join("16bit.ld").display()
        )
    }
    
    if std::env::var("CARGO_FEATURE_32BIT").unwrap() != "" {
        println!(
            "cargo:rustc-link-arg-bins=--script={}",
            local_path.join("32bit.ld").display()
        )
    }

}
