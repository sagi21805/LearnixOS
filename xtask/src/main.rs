use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{path::Path, process::Command};
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(name = "cargo xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build all stages and create image.bin
    Build {
        #[arg(long)]
        release: bool,
    },
    /// Build and launch the OS in QEMU
    Run {
        #[arg(long)]
        release: bool,
    },
    /// Run workspace-wide tests
    Test,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    // Ensure we are always running from the workspace root
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("Failed to find workspace root")?;

    println!("{:?}", root);

    sh.change_dir(root);

    match cli.command {
        Commands::Build { release } => build_os(&sh, release)?,
        Commands::Run { release } => {
            build_os(&sh, release)?;
            run_qemu()?;
        }
        Commands::Test => {
            println!("🧪 Running tests...");
            cmd!(sh, "cargo test --workspace").run()?;
        }
    }

    Ok(())
}

fn build_target(
    sh: &Shell,
    package: &str,
    target: &str,
    profile: &str,
    flags: &[&'static str],
) -> Result<()> {
    cmd!(
        sh,
        "cargo build -p {package} {profile} --target {target} {flags...}"
    )
    .run()
    .context("Failed to build first_stage")?;

    Ok(())
}

fn build_os(sh: &Shell, release: bool) -> Result<()> {
    let profile = if release { "--release" } else { "" };

    let target = "targets/16bit_target.json";

    let flags = [
        "-Z",
        "build-std=core,alloc",
        "-Z",
        "build-std-features=compiler-builtins-mem",
    ];

    let _ = build_target(
        sh,
        "first_stage",
        target,
        "--profile=bootloader",
        &flags,
    );

    let target = "targets/32bit_target.json";
    let _ = build_target(
        sh,
        "second_stage",
        target,
        "--profile=bootloader",
        &flags,
    );

    let target = "targets/64bit_target.json";
    let _ = build_target(sh, "kernel", target, "--release", &flags);

    let stage1_bin = "target/16bit_target/bootloader/first_stage";
    let stage2_bin = "target/32bit_target/bootloader/second_stage";
    let kernel = "target/64bit_target/release/kernel";
    let mut image =
        sh.read_binary_file(stage1_bin).with_context(|| {
            format!("Could not find stage1 binary at {}", stage1_bin)
        })?;

    let stage2 = sh.read_binary_file(stage2_bin).with_context(|| {
        format!("Could not find stage2 binary at {}", stage2_bin)
    })?;

    image.extend(stage2);

    let kernel = sh.read_binary_file(kernel).with_context(|| {
        format!("Could not find kernel binary at {}", kernel)
    })?;

    image.extend(kernel);
    // 4. Padding to MIN_SIZE (512KB + header/offset)
    const MIN_SIZE: usize = 515_585;
    if image.len() < MIN_SIZE {
        image.resize(MIN_SIZE, 0);
    }

    sh.write_file("image.bin", image)?;

    Ok(())
}

fn run_qemu() -> anyhow::Result<()> {
    let status = Command::new("qemu-system-x86_64")
        // Machine type
        .arg("-M")
        .arg("q35")
        .arg("-drive")
        .arg("id=disk0,file=image.bin,if=none,format=raw")
        .arg("-device")
        .arg("ide-hd,drive=disk0,bus=ide.0,rotation_rate=1")
        .arg("-monitor")
        .arg("stdio")
        .status()
        .context("Failed to execute QEMU")?;

    if !status.success() {
        anyhow::bail!("QEMU exited with error code: {}", status);
    }

    Ok(())
}
