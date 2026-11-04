use anyhow::{ensure, Context, Result};
use clap::{Args, Parser, Subcommand};
use object::{elf::SHF_ALLOC, Object, ObjectSection, SectionFlags};
use std::{
    path::Path,
    process::{Command, ExitStatus},
};
use xshell::{cmd, Shell};

/// Flags that build `core` and `alloc` for the JSON target specs.
const BUILD_STD_FLAGS: [&str; 6] = [
    "-Z",
    "build-std=core,alloc",
    "-Z",
    "build-std-features=compiler-builtins-mem",
    "-Z",
    "json-target-spec",
];

/// Flags for every clippy run. The separate target directory keeps a lint
/// from blocking a build on cargo's lock.
const CLIPPY_FLAGS: [&str; 2] =
    ["--keep-going", "--target-dir=target/clippy"];

/// The smallest `image.bin`, which is 512KB plus header and offset.
const MIN_IMAGE_SIZE: usize = 515_585;

/// Where the linked stages go, for debuggers to read their symbols.
const SYMBOLS_DIR: &str = "image-symbols";

const FIRST_STAGE: Stage = Stage {
    package: "first_stage",
    dir: "bootloader/first_stage",
    target: "16bit_target",
    profile: "bootloader",
};

const SECOND_STAGE: Stage = Stage {
    package: "second_stage",
    dir: "bootloader/second_stage",
    target: "32bit_target",
    profile: "bootloader",
};

const KERNEL: Stage = Stage {
    package: "kernel",
    dir: "kernel",
    target: "64bit_target",
    profile: "release",
};

#[derive(Parser)]
#[command(name = "cargo xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build all stages and create image.bin
    Build(BuildArgs),
    /// Build and launch the OS in QEMU
    Run(RunArgs),
    /// Lint all stages, the host-side tests and xtask with clippy
    Clippy(ClippyArgs),
}

#[derive(Args)]
struct BuildArgs {
    /// Enable debug-assertions/overflow-checks for this kernel package.
    /// Can be passed multiple times: --debug-package foo
    /// --debug-package bar
    #[arg(long = "debug-package", value_name = "PACKAGE")]
    debug_packages: Vec<String>,
}

#[derive(Args)]
struct RunArgs {
    #[command(flatten)]
    build: BuildArgs,

    /// Extra QEMU arguments, such as: -- -m 4G -s -S
    #[arg(last = true, value_name = "QEMU_ARGS")]
    qemu_args: Vec<String>,
}

#[derive(Args)]
struct ClippyArgs {
    /// Passed to cargo. rust-analyzer needs `json`.
    #[arg(long, default_value = "human")]
    message_format: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("Failed to find workspace root")?;
    sh.change_dir(root);

    match cli.command {
        Commands::Build(args) => sh.build_image(&args),
        Commands::Run(args) => {
            sh.build_image(&args.build)?;
            sh.run_qemu(&args.qemu_args)
        }
        Commands::Clippy(args) => sh.clippy(&args),
    }
}

impl BuildArgs {
    /// `--config` arguments that turn the runtime checks on for every
    /// package in `debug_packages`.
    fn debug_overrides(&self) -> Vec<String> {
        let profile = KERNEL.profile;

        self.debug_packages
            .iter()
            .flat_map(|package| {
                ["debug-assertions", "overflow-checks"].map(|setting| {
                    format!(
                        "profile.{profile}.package.{package}.\
                         {setting}=true"
                    )
                })
            })
            .flat_map(|config| ["--config".to_string(), config])
            .collect()
    }
}

#[extend::ext]
impl Shell {
    /// Builds every stage, joins their flat binaries into `image.bin`, and
    /// keeps the linked files with their symbols in `image-symbols/`.
    fn build_image(&self, args: &BuildArgs) -> Result<()> {
        self.build_stage(&FIRST_STAGE, &[])?;
        self.build_stage(&SECOND_STAGE, &[])?;
        self.build_stage(&KERNEL, &args.debug_overrides())?;

        let mut image = Vec::new();
        for stage in [&FIRST_STAGE, &SECOND_STAGE, &KERNEL] {
            let linked = self.read_binary_file(stage.elf())?;
            image.extend(
                object::File::parse(linked.as_slice())?.flat_binary()?,
            );
            self.write_file(
                format!("{SYMBOLS_DIR}/{}", stage.package),
                linked,
            )?;
        }
        if image.len() < MIN_IMAGE_SIZE {
            image.resize(MIN_IMAGE_SIZE, 0);
        }
        self.write_file("image.bin", image)?;
        Ok(())
    }

    fn build_stage(
        &self,
        stage: &Stage,
        config_overrides: &[String],
    ) -> Result<()> {
        let manifest = stage.manifest();
        let target_spec = stage.target_spec();
        let profile = stage.profile;

        cmd!(
            self,
            "
            cargo build
                --manifest-path={manifest}
                --profile={profile}
                --target={target_spec}
                {BUILD_STD_FLAGS...}
                {config_overrides...}
            "
        )
        .run()?;
        Ok(())
    }

    /// Lints each stage on its own target, then the tests on the host and
    /// xtask itself.
    ///
    /// Every lint runs even after one fails, because an editor shows only
    /// the diagnostics this command prints.
    fn clippy(&self, args: &ClippyArgs) -> Result<()> {
        let format = format!("--message-format={}", args.message_format);

        let stages = [&FIRST_STAGE, &SECOND_STAGE, &KERNEL].map(|stage| {
            let manifest = stage.manifest();
            let target_spec = stage.target_spec();
            let profile = stage.profile;

            cmd!(
                self,
                "
                cargo clippy
                    --manifest-path={manifest}
                    --profile={profile}
                    --target={target_spec}
                    {BUILD_STD_FLAGS...}
                    {CLIPPY_FLAGS...}
                    {format}
                "
            )
        });
        // The stages have panic handlers, so they can't link on the host.
        let host = cmd!(
            self,
            "
            cargo clippy
                --workspace
                --exclude=first_stage
                --exclude=second_stage
                --exclude=kernel
                --all-targets
                --target=host-tuple
                {CLIPPY_FLAGS...}
                {format}
            "
        );
        let xtask = cmd!(
            self,
            "
            cargo clippy
                --manifest-path=xtask/Cargo.toml
                --all-targets
                --target=host-tuple
                {CLIPPY_FLAGS...}
                {format}
            "
        );

        let statuses = stages
            .into_iter()
            .chain([host, xtask])
            .map(|lint| Command::from(lint).status())
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to execute cargo clippy")?;
        ensure!(
            statuses.iter().all(ExitStatus::success),
            "clippy reported errors"
        );
        Ok(())
    }

    /// Boots `image.bin` with the QEMU monitor on this terminal.
    fn run_qemu(&self, qemu_args: &[String]) -> Result<()> {
        // xshell's `run` gives the child a null stdin, which `-monitor
        // stdio` reads from, so this goes through `Command` instead.
        let mut qemu: Command = cmd!(
            self,
            "
            qemu-system-x86_64
                -M q35
                -drive id=disk0,file=image.bin,if=none,format=raw
                -device ide-hd,drive=disk0,bus=ide.0,rotation_rate=1
                -monitor stdio
                {qemu_args...}
            "
        )
        .into();

        let status = qemu.status().context("Failed to execute QEMU")?;
        ensure!(status.success(), "QEMU exited with error: {status}");
        Ok(())
    }
}

#[extend::ext]
impl object::File<'_> {
    /// The memory image of the loaded sections, starting at the lowest
    /// section address, with zeros in the gaps between them.
    ///
    /// `.text@0x10000, .data@0x10100 -> [.text, zeros, .data]`
    ///
    /// This treats section addresses as load addresses, so a linker script
    /// using `AT()` would need the segments instead.
    fn flat_binary(&self) -> Result<Vec<u8>> {
        let mut loaded = Vec::new();
        for section in self.sections() {
            let is_allocated = matches!(
                section.flags(),
                SectionFlags::Elf { sh_flags }
                    if sh_flags & u64::from(SHF_ALLOC) != 0
            );
            let data = section.data()?;
            if is_allocated && !data.is_empty() {
                loaded.push((section.address(), data));
            }
        }
        let base = loaded
            .iter()
            .map(|(address, _)| *address)
            .min()
            .unwrap_or_default();

        let mut flat = Vec::new();
        for (address, data) in loaded {
            let start = (address - base) as usize;
            let end = start + data.len();
            if flat.len() < end {
                flat.resize(end, 0);
            }
            flat[start..end].copy_from_slice(data);
        }
        Ok(flat)
    }
}

/// One binary of the boot image.
struct Stage {
    package: &'static str,
    dir: &'static str,
    target: &'static str,
    profile: &'static str,
}

impl Stage {
    fn manifest(&self) -> String { format!("{}/Cargo.toml", self.dir) }

    fn target_spec(&self) -> String {
        format!("{}/{}.json", self.dir, self.target)
    }

    /// The file the linker wrote.
    fn elf(&self) -> String {
        let Stage {
            package,
            target,
            profile,
            ..
        } = self;
        format!("target/{target}/{profile}/{package}")
    }
}
