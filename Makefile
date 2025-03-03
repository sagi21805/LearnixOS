# Target binary names
BOOT_BIN = boot.bin
RUST_TARGET = target/x86_64-unknown-none/release/bootloader
LINKER_SCRIPT = linker.ld
OUTPUT = disk_image.img
TARGET = target

# Tools
NASM = nasm
OBJCPY = objcopy
CARGO = cargo
QEMU = qemu-system-x86_64

# Default rule
all: build-rust create_img

# Step 1: Build Rust code
build-rust:
	@echo "Compiling Rust code..."
	$(CARGO) build --release --target i386-code16-boot-sector.json -Zbuild-std=core --features stage-1-2
	$(CARGO) build --release --target i686-code32-stage3.json -Zbuild-std=core --features stage-3
# Step 3: Link everything
create_img: build-rust build-asm
	$(OBJCPY) -I elf32-i386 -O binary target/i386-code16-boot-sector/release/RustOS disk_image_boot.img                    
	$(OBJCPY) -I elf32-i386 -O binary target/i686-code32-stage3/release/RustOS disk_image_stage3.img
	cat disk_image_boot.img disk_image_stage3.img > bootloader.img
# Step 4: Run in QEMU
run: all
	@echo "Running bootloader in QEMU..."
	$(QEMU) -drive format=raw,file=bootloader.img

# Clean build artifacts
clean:
	@echo "Cleaning build files..."
	$(CARGO) clean
	rm -f $(BOOT_BIN) $(OUTPUT) $(TARGET) *.img

.PHONY: all build-rust build-asm link run clean
