use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::vec;
fn build_stage(name: &str, path: &str, target: &str, profile: &str) -> Child {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let artifact_dir = PathBuf::from("build/bin");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let child = Command::new(cargo)
        .args([
            "build",
            &format!("--{}", profile),
            "--target",
            target,
            "--manifest-path",
            &format!("{}/Cargo.toml", path),
            "--target-dir",
            out_dir.as_os_str().to_str().unwrap(),
            "--out-dir",
            artifact_dir.as_os_str().to_str().unwrap(),
        ])
        .spawn()
        .expect("Failed to run cargo build");

    println!("cargo:rerun-if-changed={}/src", path);
    child
}

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let mut first_stage = build_stage(
        "first_stage",
        "stages/first_stage",
        "build/targets/16bit_target.json",
        "release",
    );
    let mut second_stage = build_stage(
        "second_stage",
        "stages/second_stage",
        "build/targets/32bit_target.json",
        "release",
    );
    let mut kernel = build_stage(
        "kernel",
        "kernel",
        "build/targets/64bit_target.json",
        &profile,
    );
    let builds = vec![&mut first_stage, &mut second_stage, &mut kernel];
    for child in builds {
        let _status = child.wait().expect("Failed to wait");
    }

    let input_dir = "build/bin"; // Change to your folder path
    let output_file = "concatenated.bin";

    let mut output = File::create(output_file).unwrap();

    let mut entries: Vec<_> = fs::read_dir(input_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .collect();

    // Optional: sort files by name
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        println!("Reading {:?}", path);

        let data = fs::read(&path).unwrap();
        output.write_all(&data).unwrap();
    }

    println!("finished building the kernel");
}
