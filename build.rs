use std::fs::{self, File};
use std::io;
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

    println!("cargo::rerun-if-changed={}/src", path);
    child
}

fn main() {
    println!("cargo::rerun-if-changed=concatenated.bin");
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

    let input_dir = PathBuf::from("build/bin"); // Change to your folder path
    let input_files = [
        input_dir.join("first_stage"),
        input_dir.join("second_stage"),
        input_dir.join("kernel"),
    ];
    let mut output = File::create("final.bin").unwrap();
    for file_name in &input_files {
        let mut input = File::open(file_name).unwrap();
        io::copy(&mut input, &mut output).unwrap();
    }

    println!("finished building the kernel");
}
