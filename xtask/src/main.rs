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
        Commands::Build { release } => sh.build_os(release)?,
        Commands::Run { release } => {
            sh.build_os(release)?;
            run_qemu()?;
        }
    }

    Ok(())
}

#[extend::ext]
impl Shell {
    fn build_target(
        &self,
        manifest: &str,
        target: &str,
        profile: &str,
        flags: &[&'static str],
    ) -> Result<()> {
        cmd!(
            self,
            "
            cargo build
                --manifest-path={manifest}
                --profile={profile}
                --target={target}
             {flags...}
            "
        )
        .run()
        .context("Failed to build first_stage")?;

        Ok(())
    }

    fn build_os(&self, release: bool) -> Result<()> {
        let flags = [
            "-Z",
            "build-std=core,alloc",
            "-Z",
            "build-std-features=compiler-builtins-mem",
            "-Z",
            "json-target-spec",
        ];

        self.build_target(
            "bootloader/first_stage/Cargo.toml",
            "bootloader/first_stage/16bit_target.json",
            "release",
            &flags,
        )?;

        self.build_target(
            "bootloader/second_stage/Cargo.toml",
            "bootloader/second_stage/32bit_target.json",
            "release",
            &flags,
        )?;

        self.build_target(
            "kernel/Cargo.toml",
            "kernel/64bit_target.json",
            "release",
            &flags,
        )?;

        let stage1_bin = "bootloader/first_stage/target/16bit_target/\
                          release/first_stage";
        let stage2_bin = "bootloader/second_stage/target/32bit_target/\
                          release/second_stage";
        let kernel = "kernel/target/64bit_target/release/kernel";
        let mut image =
            self.read_binary_file(stage1_bin).with_context(|| {
                format!("Could not find stage1 binary at {}", stage1_bin)
            })?;

        let stage2 =
            self.read_binary_file(stage2_bin).with_context(|| {
                format!("Could not find stage2 binary at {}", stage2_bin)
            })?;

        image.extend(stage2);

        let kernel = self.read_binary_file(kernel).with_context(|| {
            format!("Could not find kernel binary at {}", kernel)
        })?;

        image.extend(kernel);
        // 4. Padding to MIN_SIZE (512KB + header/offset)
        const MIN_SIZE: usize = 515_585;
        if image.len() < MIN_SIZE {
            image.resize(MIN_SIZE, 0);
        }

        self.write_file("image.bin", image)?;

        Ok(())
    }
}

fn run_qemu() -> anyhow::Result<()> {
    let status = Command::new("qemu-system-x86_64")
        // Machine type
        .arg("-M")
        .arg("q35")
        .arg("-m")
        .arg("4G")
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
