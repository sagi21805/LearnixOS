use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    
    if std::env::var("CARGO_FEATURE_STAGE_1_2").unwrap() != "" {
        println!(
            "cargo:rustc-link-arg-bins=--script={}",
            local_path.join("linker.ld").display()
        )
    }
    
    if std::env::var("CARGO_FEATURE_STAGE_3").unwrap() != "" {
        println!(
            "cargo:rustc-link-arg-bins=--script={}",
            local_path.join("stage_3.ld").display()
        )
    }

}
