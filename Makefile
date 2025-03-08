# Tools
OBJCPY = objcopy
CARGO = cargo
QEMU = qemu-system-x86_64
OUTPUT = bootloader.img

# Targets
16BIT = 16bit_target
32BIT = 32bit_target

# Default rule
all: build-rust create_img

# Step 1: Build Rust code
build-rust:
	@echo "Compiling Rust code..."
	$(CARGO) build --release --target $(16BIT).json -Zbuild-std=core --features 16bit
	$(CARGO) build --release --target $(32BIT).json -Zbuild-std=core --features 32bit

# Step 3: Link everything
create_img: build-rust build-asm
	$(OBJCPY) -I elf32-i386 -O binary target/$(16BIT)/release/RustOS 16bit.img                    
	$(OBJCPY) -I elf32-i386 -O binary target/$(32BIT)/release/RustOS 32bit.img
	cat 16bit.img 32bit.img > $(OUTPUT)
# rm 16bit.img 32bit.img
# Step 4: Run in QEMU
run: all
	@echo "Running bootloader in QEMU..."
	$(QEMU) -drive format=raw,file=$(OUTPUT)

# Clean build artifacts
clean:
	@echo "Cleaning build files..."
	$(CARGO) clean
	rm -f $(BOOT_BIN) $(OUTPUT) $(TARGET) *.img

.PHONY: all build-rust build-asm link run clean
