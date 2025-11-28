use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

fn build_stage(path: &str, target: &str, profile: &str) -> ExitStatus {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let artifact_dir = PathBuf::from("bin");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    let status = Command::new(cargo)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .args([
            "build",
            "--color=always",
            &format!("--{}", profile),
            "--target",
            target,
            "--manifest-path",
            &format!("{}/Cargo.toml", path),
            "--target-dir",
            out_dir.as_os_str().to_str().unwrap(),
            "--artifact-dir",
            artifact_dir.as_os_str().to_str().unwrap(),
        ])
        .status()
        .unwrap_or_else(|_| {
            panic!("Failed to run build script for {}", path)
        });
    println!("cargo::rerun-if-changed={}/src", path);

    if !status.success() {
        panic!("Cargo build failed for {}", path);
    }

    status
}

fn main() -> io::Result<()> {
    // Force rerun every time
    println!("cargo::rerun-if-changed=nonexistent.file");

    println!("cargo::rerun-if-changed=image.bin");

    let profile = std::env::var("PROFILE").unwrap();

    // Run each stage and wait immediately
    build_stage(
        "../kernel/stages/first_stage",
        "targets/16bit_target.json",
        "release",
    );
    build_stage(
        "../kernel/stages/second_stage",
        "targets/32bit_target.json",
        "release",
    );
    build_stage("../kernel", "targets/64bit_target.json", &profile);

    // Combine binaries into one image
    let input_dir = PathBuf::from("bin");
    let input_files = [
        input_dir.join("first_stage"),
        input_dir.join("second_stage"),
        input_dir.join("kernel"),
    ];
    let mut output = File::create("image.bin").unwrap();
    for file_name in &input_files {
        let mut input = File::open(file_name).unwrap();
        io::copy(&mut input, &mut output).unwrap();
    }
    const MIN_SIZE: u64 = 515_585;
    let current_size = output.metadata()?.len();
    if current_size < MIN_SIZE {
        // Seek to the target size - 1
        output.seek(SeekFrom::Start(MIN_SIZE - 1))?;
        // Write a single zero byte at the end
        output.write_all(&[0])?;
    }
    println!("finished building the kernel");
    Ok(())
}
