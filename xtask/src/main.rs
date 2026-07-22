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

        /// Enable debug-assertions/overflow-checks/debug-info for this
        /// package, even when the rest of the build is in
        /// --release mode. Can be passed multiple times:
        /// --debug-package foo --debug-package bar
        #[arg(long = "debug-package", value_name = "PACKAGE")]
        debug_packages: Vec<String>,
    },
    /// Build and launch the OS in QEMU
    Run {
        #[arg(long)]
        release: bool,

        #[arg(long = "debug-package", value_name = "PACKAGE")]
        debug_packages: Vec<String>,
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
        Commands::Build {
            release,
            debug_packages,
        } => sh.build_os(release, &debug_packages)?,
        Commands::Run {
            release,
            debug_packages,
        } => {
            sh.build_os(release, &debug_packages)?;
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
        extra_flags: &[&'static str],
        debug_packages: &[String],
    ) -> Result<()> {
        // Build --config overrides for any packages that should keep
        // debug-assertions/overflow-checks/debug-info even in a release
        // profile.
        let overrides: Vec<String> = debug_packages
            .iter()
            .flat_map(|pkg| {
                [
                    format!(
                        "profile.{profile}.package.{pkg}.\
                         debug-assertions=true"
                    ),
                    format!(
                        "profile.{profile}.package.{pkg}.\
                         overflow-checks=true"
                    ),
                    format!("profile.{profile}.package.{pkg}.debug=true"),
                ]
            })
            .flat_map(|kv| ["--config".to_string(), kv])
            .collect();

        cmd!(
            self,
            "
            cargo build
                --manifest-path={manifest}
                --profile={profile}
                --target={target}
             {extra_flags...}
             {overrides...}
            "
        )
        .run()
        .with_context(|| format!("Failed to build {manifest}"))?;
        Ok(())
    }

    fn build_os(
        &self,
        release: bool,
        debug_packages: &[String],
    ) -> Result<()> {
        let profile = if release { "release" } else { "dev" };
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
            profile,
            &flags,
            debug_packages,
        )?;
        self.build_target(
            "bootloader/second_stage/Cargo.toml",
            "bootloader/second_stage/32bit_target.json",
            profile,
            &flags,
            debug_packages,
        )?;
        self.build_target(
            "kernel/Cargo.toml",
            "kernel/64bit_target.json",
            profile,
            &flags,
            debug_packages,
        )?;

        // NOTE: profile "dev" -> target dir is `debug`, not the profile
        // name itself.
        let profile_dir = if release { "release" } else { "debug" };
        let stage1_bin =
            format!("target/16bit_target/{profile_dir}/first_stage");
        let stage2_bin =
            format!("target/32bit_target/{profile_dir}/second_stage");
        let kernel_bin =
            format!("target/64bit_target/{profile_dir}/kernel");

        let mut image =
            self.read_binary_file(&stage1_bin).with_context(|| {
                format!("Could not find stage1 binary at {}", stage1_bin)
            })?;
        let stage2 =
            self.read_binary_file(&stage2_bin).with_context(|| {
                format!("Could not find stage2 binary at {}", stage2_bin)
            })?;
        image.extend(stage2);
        let kernel =
            self.read_binary_file(&kernel_bin).with_context(|| {
                format!("Could not find kernel binary at {}", kernel_bin)
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
        // .arg("-m")
        // .arg("4G")
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
